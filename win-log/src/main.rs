#[macro_use]
extern crate lazy_static;

use windows::Win32::System::EventLog;
use windows::Win32::Security;
use windows::Win32::System::Registry;
use windows::core;
use std::ffi::c_void;
use std::collections::BTreeMap;
use windows::Win32::Foundation;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
mod configuration;
mod common;
mod extractors;

use common::{EventBookmark, ListenerConfiguration, ULoggerError, to_pwstr};


fn main() {
    // https://stackoverflow.com/questions/57663035/how-to-create-a-subscription-for-events-in-windows-using-rust-and-the-winapi-cra
    //https://docs.microsoft.com/en-us/windows/win32/wes/subscribing-to-events

    
}


fn get_listeners(working_path : &str) -> Result<Vec<ListenerConfiguration>,ULoggerError> {
    let listener_path = Path::new(working_path).join("listeners.json");
    if !listener_path.exists() {
        return Err(ULoggerError::InvalidPath(format!("Invalid path {}", working_path)))
    }
    let listener_conf_content = match fs::read_to_string(&listener_path) {
        Ok(listener_conf_content) => listener_conf_content,
        Err(_) => return Err(ULoggerError::InvalidPath(format!("Invalid path {:?}", &listener_path)))
    };

    match serde_json::from_str(&listener_conf_content) {
        Ok(content) => Ok(content),
        Err(_) => Err(ULoggerError::InvalidConfigFile(format!("Invalid config file {:?}", &listener_path)))
    }
}

fn get_listeners_or_default() -> Result<Vec<ListenerConfiguration>,ULoggerError> {
    match get_listeners(configuration::WORKING_DIRECTORY) {
        Ok(listeners) => Ok(listeners),
        Err(_) => {
            let mut listener_list = Vec::new();
            for listener in configuration::LISTENER_LIST.iter() {
                listener_list.push(ListenerConfiguration{
                    bookmark : None,
                    query : listener.1.to_string(),
                    channel : listener.0.to_string()
                })
            }
            Ok(listener_list)
        }
    }
}

fn create_event_subscription(listener : ListenerConfiguration) -> async_std::task::JoinHandle<()> {
    async_std::task::spawn(async {
        //Unsafe code inside -> Subscribe to events and send them to a parser and an output
    })
}
unsafe fn events_from_result_set(event_handle : isize) -> Vec<String> {
    let mut result_set : [isize; 512] = [0; 512];
    let mut returned : u32 = 0;
    if EventLog::EvtNext(event_handle, 512,result_set.as_mut_ptr(), 20, 0,&mut returned) == Foundation::BOOL::from(false) {
        let status = Foundation::GetLastError();
        if status == Foundation::ERROR_NO_MORE_ITEMS {
                panic!("EvtNext failed with {}", status);
        }
    }
    let mut to_return_evts : Vec<String> = Vec::with_capacity(returned as usize);
    let buffersize : u32 = 4096;
    let mut bufferused : u32 = 0;
    let mut propertycount : u32 = 0;
    let mut buffer = Vec::<u8>::with_capacity(buffersize as usize);
    for i in 0..returned {
        let evt_txt = EventLog::EvtRender(0, result_set[i as usize], EventLog::EvtRenderEventXml as u32, buffersize, buffer.as_mut_ptr() as *mut std::ffi::c_void, &mut bufferused, &mut propertycount);
        if evt_txt == Foundation::BOOL::from(false) {
            println!("Error creating string event");
        }
        println!("bufferused = {:?}",bufferused);
        let slice = std::slice::from_raw_parts(buffer.as_mut_ptr() as _, (bufferused/2) as usize);
        let mut new_str = String::from_utf16_lossy(slice);
        new_str.remove(new_str.len() - 1);
        println!("{:?}", new_str);  
        to_return_evts.push(new_str);
    }

    return to_return_evts
}

#[test]
fn it_works() -> Result<(),ULoggerError>{
    return Ok(());
    let listeners = get_listeners_or_default()?;
    unsafe {
        let callback: EventLog::EVT_SUBSCRIBE_CALLBACK = None;
        let flags = Security::SECURITY_ATTRIBUTES::default();
        let signal_event: Foundation::HANDLE = windows::Win32::System::Threading::CreateEventA(&flags, true, true, Foundation::PSTR(b"".as_ptr() as _));

        let context = std::ptr::null_mut();

        if signal_event.is_invalid() {
            println!("Invalid SignalEvent")
        }
        let v: Vec<u16> = to_pwstr("Security");
        let channel_path: Foundation::PWSTR = Foundation::PWSTR(v.as_ptr() as _);
        let v: Vec<u16> = to_pwstr("*");
        let query: Foundation::PWSTR = Foundation::PWSTR(v.as_ptr() as _);
        let bookmark : isize = 0;
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
        }else if error == Foundation::ERROR_EVT_CHANNEL_NOT_FOUND {
            println!("Channel not found");
        }
        else if error == Foundation::ERROR_EVT_CHANNEL_NOT_FOUND {
            println!("Channel not found");
        }else if error == Foundation::ERROR_EVT_INVALID_QUERY {
            println!("Channel not found");
        }else if error == 0 {
            //No error

        }else{
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
