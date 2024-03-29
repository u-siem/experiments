#[macro_use]
extern crate lazy_static;

use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use windows::Win32::Foundation;
use windows::Win32::Security;
use windows::Win32::System::EventLog::{self, EvtUpdateBookmark};
mod common;
mod configuration;
mod extractors;

use common::{to_pwstr, EventBookmark, EvtListenerConfiguration, ULoggerError, ListenerConfiguration};
use crossbeam_channel::{Receiver, Sender, TrySendError};
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    let listeners = match get_listeners_or_default() {
        Ok(listeners) => listeners,
        Err(e) => match e {
            ULoggerError::InvalidConfigFile(_) => return,
            ULoggerError::InvalidPath(_) => return,
            _ => return
        },
    };
    let (sender, receiver): (Sender<String>, Receiver<String>) = crossbeam_channel::bounded(2048);
    let handlers = match start_listeners(listeners, sender) {
        Ok(handlers) => handlers,
        Err(_) => {
            panic!("Cannot start program")
        }
    };
    std::thread::sleep(Duration::from_millis(100));
    let mut out_file = fs::File::create("./file_log.jsonl").unwrap();

    let mut tries = 0;
    loop {
        match receiver.try_recv() {
            Ok(msg) => {
                // Parse or something
                println!("New event: \n{}",msg);
                let _ = out_file.write(msg.as_bytes());
                let _ = out_file.write(b"\n");
            }
            Err(_) => {
                if tries > 1000 {
                    break;
                }
                tries += 1;
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
    
    stop_all_running_listeners();
    
    async_std::task::block_on(async {
        for handle in handlers {
            handle.await;
        }
    });
    configuration::save_running_configuration();
}


fn stop_all_running_listeners() {
    match configuration::RUNNING_LISTENER_CONFIGURATION.lock() {
        Ok(guard) => {
            let mut i = 0;
            for listener in guard.iter() {
                match listener.lock() {
                    Ok(listener_config) => {
                        match *listener_config {
                            ListenerConfiguration::Event(ref evt_config) => {
                                evt_config.stop.store(true, Ordering::Relaxed);
                            },
                            ListenerConfiguration::File(ref file_config) => {
                                file_config.stop.store(true, Ordering::Relaxed);
                            }
                        }
                    }
                    Err(_) => {
                        println!("Cannot aquire mutex {}", i);
                    }
                }
                i += 1;
            }
        }
        Err(_) => {
        }
    }
}

fn start_listeners(
    listeners: Vec<ListenerConfiguration>,
    log_sender: Sender<String>,
) -> Result<Vec<async_std::task::JoinHandle<()>>, ULoggerError> {
    let mut handlers = Vec::with_capacity(128);
    match configuration::RUNNING_LISTENER_CONFIGURATION.lock() {
        Ok(mut guard) => {
            for listener in listeners {
                let shared_listener = Arc::new(Mutex::new(listener));
                let listener_reference = Arc::clone(&shared_listener);
                (*guard).push(shared_listener);
                let handler = match spawn_evt_listener_task(listener_reference, log_sender.clone()) {
                    Ok(handle) => handle,
                    Err(_) => {continue;}
                };
                handlers.push(handler);
            }
        }
        Err(_) => {
            return Err(ULoggerError::InvalidConfigFile(format!(
                "Cannot aquire running listener mutex"
            )))
        }
    }

    Ok(handlers)
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
                            println!("No events");
                        }else{
                            times_without_events = 0;
                            for evt in events {
                                let mut msg = evt;
                                loop {
                                    match log_sender.try_send(msg) {
                                        Ok(_) => break,
                                        Err(e) => {
                                            match e {
                                                TrySendError::Disconnected(_) => {
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
    println!("Created async task");
    Ok(handle)
}

fn get_listeners(path: &str) -> Result<Vec<ListenerConfiguration>, ULoggerError> {
    let working_path = Path::new(path);
    if !working_path.exists() {
        match fs::create_dir(working_path) {
            Ok(()) => {}
            Err(_) => {
                return Err(ULoggerError::InvalidPath(format!(
                    "Cannot create working path {}",
                    path
                )))
            }
        }
    }
    let listener_path = working_path.join("listeners.json");
    if !listener_path.exists() {
        return Err(ULoggerError::InvalidPath(format!("Invalid path {}", path)));
    }
    let listener_conf_content = match fs::read_to_string(&listener_path) {
        Ok(listener_conf_content) => listener_conf_content,
        Err(_) => {
            return Err(ULoggerError::InvalidPath(format!(
                "Invalid path {:?}",
                &listener_path
            )))
        }
    };
    match serde_json::from_str(&listener_conf_content) {
        Ok(content) => Ok(content),
        Err(e) => {
            println!("{}",e);
            Err(ULoggerError::InvalidConfigFile(format!(
                "Invalid config file {:?}",
                &listener_path
            )))
        },
    }
}

fn get_listeners_or_default() -> Result<Vec<ListenerConfiguration>, ULoggerError> {
    match get_listeners(configuration::WORKING_DIRECTORY) {
        Ok(listeners) => Ok(listeners),
        Err(_) => {
            let mut listener_list = Vec::new();
            for listener in configuration::LISTENER_LIST.iter() {
                listener_list.push(ListenerConfiguration::Event(EvtListenerConfiguration {
                    bookmark: None,
                    query: listener.1.to_string(),
                    channel: listener.0.to_string(),
                    stop: Arc::new(AtomicBool::new(false)),
                }));
            }
            Ok(listener_list)
        }
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

#[test]
fn it_works() -> Result<(), ULoggerError> {
    return Ok(());
    let listeners = get_listeners_or_default()?;
    unsafe {
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
        let v: Vec<u16> = to_pwstr("Security");
        let channel_path: Foundation::PWSTR = Foundation::PWSTR(v.as_ptr() as _);
        let v: Vec<u16> = to_pwstr("*");
        let query: Foundation::PWSTR = Foundation::PWSTR(v.as_ptr() as _);
        let bookmark: isize = 0;
        //TODO: Bookmark
        let event_handle = EventLog::EvtSubscribe(
            0,
            signal_event,
            channel_path,
            query,
            bookmark,
            context,
            callback,
            EventLog::EvtSubscribeStartAtOldestRecord as u32,
        );
        println!("Valid event_handle {}", event_handle);
        if event_handle != 0 {
            println!("Valid event_handle")
        }

        let error = Foundation::GetLastError();

        if error == Foundation::ERROR_EVT_INVALID_CHANNEL_PATH {
            println!("InvalidChannelPath");
        } else if error == Foundation::ERROR_EVT_CHANNEL_NOT_FOUND {
            println!("Channel not found");
        } else if error == Foundation::ERROR_EVT_CHANNEL_NOT_FOUND {
            println!("Channel not found");
        } else if error == Foundation::ERROR_EVT_INVALID_QUERY {
            println!("Channel not found");
        } else if error == 0 {
            //No error
        } else {
            panic!("ERROR {:?}", &error);
        }
        for _ in 0..100 {
            events_from_result_set(event_handle);
        }

        // Close
        EventLog::EvtClose(event_handle);
    } //unsafe end
    Ok(())
}
