use std::{sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, time::Duration};
use async_std::channel::{Sender, TrySendError};

use windows::Win32::{System::EventLog::{self, EvtUpdateBookmark}, Foundation, Security};

use crate::common::{ListenerConfiguration, ULoggerError, EventBookmark, to_pwstr};

pub struct WinEvtListener {
    config : Vec<Arc<Mutex<ListenerConfiguration>>>
}

impl WinEvtListener {

    pub fn init() -> Self {
        Self {
            config : Vec::new()
        }
    }

    pub fn add_configuration(&mut self, config : Arc<Mutex<ListenerConfiguration>>) {
        match config.lock() {
            Ok(config_guard) => {
                if let ListenerConfiguration::Event(_) = *config_guard {
                    self.config.push(config.clone());
                }
            },
            Err(_) => {
                //TODO
            }
        }
    }

    pub fn start_listeners(&self,log_sender: Sender<String>,
    ) -> Result<Vec<async_std::task::JoinHandle<()>>, ULoggerError> {
        let mut handlers = Vec::with_capacity(128);
        for config in &self.config {
            match spawn_evt_listener_task(config.clone(), log_sender.clone()) {
                Ok(handler) => {
                    handlers.push(handler);
                },
                Err(_) => {}
            }
        }
    
        Ok(handlers)
    }
}






unsafe fn subscribe_with_bookmark(
    event_handle: isize,
    bookmark: isize,
) -> Result<Vec<String>, ULoggerError> {
    let mut result_set: [isize; 512] = [0; 512];
    let mut returned: u32 = 0;
    if EventLog::EvtNext(
        event_handle,
        512,
        result_set.as_mut_ptr(),
        20,
        0,
        &mut returned,
    ) == Foundation::BOOL::from(false)
    {
        let status = Foundation::GetLastError();
        if status == Foundation::ERROR_NO_MORE_ITEMS {
            return Ok(Vec::new());
        }
    }
    let mut to_return_evts: Vec<String> = Vec::with_capacity(returned as usize);
    let buffersize: u32 = 4096;
    let mut bufferused: u32 = 0;
    let mut propertycount: u32 = 0;
    let mut buffer = Vec::<u8>::with_capacity(buffersize as usize);
    for i in 0..returned {
        let evt_txt = EventLog::EvtRender(
            0,
            result_set[i as usize],
            EventLog::EvtRenderEventXml as u32,
            buffersize,
            buffer.as_mut_ptr() as *mut std::ffi::c_void,
            &mut bufferused,
            &mut propertycount,
        );
        if evt_txt == Foundation::BOOL::from(false) {
            println!("Error creating string event");
        }
        
        EvtUpdateBookmark(bookmark, result_set[i as usize]);
        let slice = std::slice::from_raw_parts(buffer.as_mut_ptr() as _, (bufferused / 2) as usize);
        let mut new_str = String::from_utf16_lossy(slice);
        new_str.remove(new_str.len() - 1);
        to_return_evts.push(new_str);
    }
    Ok(to_return_evts)
}

fn spawn_evt_listener_task(
    listener: Arc<Mutex<ListenerConfiguration>>,
    log_sender: Sender<String>,
) -> Result<async_std::task::JoinHandle<()>, ULoggerError> {
    let (bookmark, query, channel, stop_task): (
        Option<EventBookmark>,
        String,
        String,
        Arc<AtomicBool>,
    ) = match listener.lock() {
        Ok(guard) => {
            match *guard {
                ListenerConfiguration::Event(ref evt_config) => {
                    (evt_config.bookmark.clone(),
                    evt_config.query.clone(),
                    evt_config.channel.clone(),
                    evt_config.get_stop_listener(),
                    )
                },
                _ => return Err(ULoggerError::InvalidConfigFile(format!("Invalid configuration. Must be EventListener")))
            }
        },
        Err(_) => {
            return Err(ULoggerError::InvalidConfigFile(format!(
                "Cannot aquire running listener mutex"
            )))
        }
    };
    let (event_handle, bookmark) = unsafe {
        let bookmark = {
            match bookmark {
                Some(bookmark) => {
                    let v: Vec<u16> = to_pwstr(&bookmark.xml);
                    let bookmark_xml: Foundation::PWSTR = Foundation::PWSTR(v.as_ptr() as _);
                    let bookmark = EventLog::EvtCreateBookmark(bookmark_xml);
                    println!("Old bookmark");
                    if bookmark == 0 {
                        return Err(ULoggerError::InvalidConfigFile(format!(
                            "Cannot create bookmark"
                        )));
                    }
                    bookmark
                }
                None => {
                    println!("New bookmark");
                    // Create bookmark
                    let bookmark = EventLog::EvtCreateBookmark(Foundation::PWSTR(0 as _));
                    if bookmark == 0 {
                        return Err(ULoggerError::InvalidConfigFile(format!(
                            "Cannot create bookmark"
                        )));
                    }
                    bookmark
                }
            }
        };
        let callback: EventLog::EVT_SUBSCRIBE_CALLBACK = None;
        let flags = Security::SECURITY_ATTRIBUTES::default();
        let signal_event: Foundation::HANDLE = windows::Win32::System::Threading::CreateEventA(
            &flags,
            true,
            true,
            Foundation::PSTR(b"".as_ptr() as _),
        );

        let context = std::ptr::null_mut();

        if signal_event.is_invalid() {
            println!("Invalid SignalEvent")
        }
        let v: Vec<u16> = to_pwstr(&channel);
        let channel_path: Foundation::PWSTR = Foundation::PWSTR(v.as_ptr() as _);
        let v: Vec<u16> = to_pwstr(&query);
        let query: Foundation::PWSTR = Foundation::PWSTR(v.as_ptr() as _);

        let event_handle = EventLog::EvtSubscribe(
            0,
            signal_event,
            channel_path,
            query,
            bookmark,
            context,
            callback,
            EventLog::EvtSubscribeStartAfterBookmark as u32,
        );
        if event_handle != 0 {
            println!("Valid event_handle")
        }

        let error = Foundation::GetLastError();

        if error == Foundation::ERROR_EVT_INVALID_CHANNEL_PATH {
            EventLog::EvtClose(bookmark);
            return Err(ULoggerError::EventChannelNotFound(format!("Invalid channel: {}", channel)));
        } else if error == Foundation::ERROR_EVT_CHANNEL_NOT_FOUND {
            EventLog::EvtClose(bookmark);
            return Err(ULoggerError::EventChannelNotFound(format!("Channel not found: {}", channel)));
        } else if error == Foundation::ERROR_EVT_INVALID_QUERY {
            EventLog::EvtClose(bookmark);
            return Err(ULoggerError::EventQueryInvalid(format!("Invalid query: {}", channel)));
        } else if error == 0 {
            //No error
        } else {
            panic!("ERROR {:?}", &error);
        }
        println!("Tenemos event_handle {} y bookmark {}", event_handle, bookmark);
        (event_handle, bookmark)
    };

    let handle = async_std::task::spawn(async move {
        //Unsafe code inside -> Subscribe to events and send them to a parser and an output
        let mut times_without_events = 0;
        unsafe {
            println!("Loop start for handle {}", event_handle);
            loop {
                let events = subscribe_with_bookmark(event_handle, bookmark);
                match events {
                    Ok(events) => {
                        if events.len() == 0 {
                            
                            times_without_events += 1;
                            async_std::task::yield_now().await;
                            if times_without_events > 10 {
                                async_std::task::sleep(Duration::from_millis(100)).await;
                            }
                        }else{
                            times_without_events = 0;
                            for evt in events {
                                let mut msg = evt;
                                loop {
                                    match log_sender.try_send(msg) {
                                        Ok(_) => break,
                                        Err(e) => {
                                            match e {
                                                TrySendError::Closed(_) => {
                                                    println!("Disconected!");
                                                    stop_task.store(true, Ordering::Relaxed);
                                                    break;
                                                },
                                                TrySendError::Full(evt) => {
                                                    msg = evt;
                                                    println!("Sleep");
                                                    async_std::task::sleep(Duration::from_millis(100)).await;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        panic!("Errpr: {:?}", e);
                    }
                }
                if stop_task.load(Ordering::Relaxed) {
                    println!("Listener stopped");
                    // Close
                    EventLog::EvtClose(event_handle);
                    let bookmarkxml = render_bookmark(bookmark);
                    match bookmarkxml {
                        Ok(bookmarkxml) => {
                            println!("{}" , bookmarkxml);
                            match listener.lock() {
                                Ok(mut guard) => {
                                    match *guard {
                                        ListenerConfiguration::Event(ref mut evt_config) => {
                                            evt_config.bookmark = Some(EventBookmark { xml: bookmarkxml });
                                        },
                                        _ => {}
                                    }                                    
                                }
                                Err(_) => {
                                    println!("Cannot save bookmark")
                                }
                            };
                        }
                        Err(_) => {}
                    };
                    EventLog::EvtClose(bookmark);
                    return;
                }
            }
        }
    });
    Ok(handle)
}



fn extract_record_id(msg : &str) -> &str {
    match msg.find("EventRecordID") {
        Some(record_id_pos) => {
            let recordId2 = &msg[record_id_pos + 14..].find("<").unwrap();
            &msg[record_id_pos + 14..record_id_pos + 14 + *recordId2]
        },
        None => "0"
    }
    
}


unsafe fn render_bookmark(handler: isize) -> Result<String, ULoggerError> {
    let buffersize: u32 = 4096;
    let mut bufferused: u32 = 0;
    let mut propertycount: u32 = 0;
    let mut buffer = Vec::<u8>::with_capacity(buffersize as usize);
    let evt_txt = EventLog::EvtRender(
        0,
        handler,
        EventLog::EvtRenderBookmark as u32,
        buffersize,
        buffer.as_mut_ptr() as *mut std::ffi::c_void,
        &mut bufferused,
        &mut propertycount,
    );
    if evt_txt == Foundation::BOOL::from(false) {
        return Err(ULoggerError::InvalidConfigFile(
            "Bookmark not rendered".to_string(),
        ));
    }
    let slice = std::slice::from_raw_parts(buffer.as_mut_ptr() as _, (bufferused / 2) as usize);
    let mut new_str = String::from_utf16_lossy(slice);
    new_str.remove(new_str.len() - 1);
    Ok(new_str)
}

unsafe fn events_from_result_set(event_handle: isize) -> Vec<String> {
    let mut result_set: [isize; 512] = [0; 512];
    let mut returned: u32 = 0;
    if EventLog::EvtNext(
        event_handle,
        512,
        result_set.as_mut_ptr(),
        20,
        0,
        &mut returned,
    ) == Foundation::BOOL::from(false)
    {
        let status = Foundation::GetLastError();
        if status == Foundation::ERROR_NO_MORE_ITEMS {
            panic!("EvtNext failed with {}", status);
        }
    }
    let mut to_return_evts: Vec<String> = Vec::with_capacity(returned as usize);
    let buffersize: u32 = 4096;
    let mut bufferused: u32 = 0;
    let mut propertycount: u32 = 0;
    let mut buffer = Vec::<u8>::with_capacity(buffersize as usize);
    for i in 0..returned {
        let evt_txt = EventLog::EvtRender(
            0,
            result_set[i as usize],
            EventLog::EvtRenderEventXml as u32,
            buffersize,
            buffer.as_mut_ptr() as *mut std::ffi::c_void,
            &mut bufferused,
            &mut propertycount,
        );
        if evt_txt == Foundation::BOOL::from(false) {
            println!("Error creating string event");
        }
        let slice = std::slice::from_raw_parts(buffer.as_mut_ptr() as _, (bufferused / 2) as usize);
        let mut new_str = String::from_utf16_lossy(slice);
        new_str.remove(new_str.len() - 1);
        println!("{:?}", new_str);
        to_return_evts.push(new_str);
    }

    return to_return_evts;
}