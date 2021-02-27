use bindings::{
    windows::win32::windows_event_log::{EvtSubscribe,EVT_SUBSCRIBE_CALLBACK, EVT_SUBSCRIBE_NOTIFY_ACTION, EVT_SUBSCRIBE_FLAGS,EvtOpenChannelEnum,EvtNextChannelPath,EVT_RENDER_FLAGS, EvtRender},
    windows::data::xml::dom::XmlDocument,
    windows::win32::system_services::HANDLE,
    windows::ui::notifications::{ToastNotification, ToastNotifier, ToastNotificationManager, ToastTemplateType},
    windows::win32::debug::GetLastError
};
use std::ffi::c_void;
use std::collections::BTreeMap;

fn main() {
    /*
    unsafe {
        let notifier = ToastNotificationManager::create_toast_notifier_with_id("MySupperApp").unwrap();
        let mut xml = ToastNotificationManager::get_template_content(ToastTemplateType::ToastImageAndText01).unwrap();
        xml.load_xml("<visual><binding template=\"ToastGeneric\"><text>DNS Alert...</text><text>We noticed that you are near Wasaki. Thomas left a 5 star rating after his last visit, do you want to try it?</text></binding></visual>").unwrap();
        let data = ToastNotification::create_toast_notification(xml).unwrap();
        
        notifier.show(data).unwrap();
    };*/
    /*
    unsafe {
        let mut xml = XmlDocument::new().unwrap();
        xml.load_xml("<visual><binding template=\"ToastGeneric\"><text>DNS Alert...</text><text>We noticed that you are near Wasaki. Thomas left a 5 star rating after his last visit, do you want to try it?</text></binding></visual>").unwrap();
        let data = ToastNotification::create_toast_notification(xml).unwrap();
        let manager = ToastNotificationManager::get_default().unwrap();
        let mut notifier = manager.create_toast_notifier_with_id("MySupperApp").unwrap();
        notifier.show(data).unwrap();
    };*/

    unsafe {
        
        let mut session : isize = 0;
        let mut signal_event : isize = 0;

        let mut bookmark : isize = 0;
        let mut context = std::ptr::null_mut();
        let txt_v = to_utf16("Windows Powershell");
        let channel : &[u16] = &txt_v;
        let query = to_utf16("*");
        let query : &[u16] = &query;
        let flags : u32 = EVT_SUBSCRIBE_FLAGS::EvtSubscribeToFutureEvents.0 as u32;
        let event_handle = EvtSubscribe(
            session,
            HANDLE(0 as isize),
            channel.as_ptr(),
            query.as_ptr(),
            bookmark,
            context,
            Some(event_callback),
            flags
        );
        //std::thread::sleep(std::time::Duration::from_secs(10));
        println!("Error: {:?}", &GetLastError());
        let mut channels = EvtOpenChannelEnum(session,0);
        println!("Channels: {:?}", &channels);
        let mut buffer: [u16; 10000] = [0; 10000];
        let mut used_buffer = 0;
        EvtNextChannelPath(channels, 10000,buffer.as_mut_ptr(), &mut used_buffer);
        let text = String::from_utf16_lossy(&buffer[..used_buffer as usize]);
        println!("{:?}", &text);
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    } //unsafe end*/
    
}

unsafe fn parse_event(event_handler : isize) -> String {
    let buffer_size = 65_000;
    let mut property_count = 0;
    let mut buffer: [u16; 65_000] = [0; 65_000];
    let mut used_buffer = 0;
    let a = EvtRender(0 as isize, event_handler, EVT_RENDER_FLAGS::EvtRenderEventXml.0 as u32, buffer_size as u32, buffer.as_mut_ptr() as *mut c_void, &mut used_buffer, &mut property_count);
    println!("{}",property_count);
    return String::from_utf16_lossy(&buffer[..((used_buffer/2) as usize)]);
}

extern "system" fn event_callback(action : EVT_SUBSCRIBE_NOTIFY_ACTION, pContext : *mut c_void, hEvent: isize) -> u32 {
    if action == EVT_SUBSCRIBE_NOTIFY_ACTION::EvtSubscribeActionError {
        println!("Error: {:?}", action);
    }else{
        unsafe {
            let evnt = parse_event(hEvent);
            println!("{:?}",evnt);
            println!("{:?}",evnt.len());
        }
        
    }
    0
}

fn to_utf16(txt: &str) ->Vec<u16>  {
    let mut v: Vec<u16> = txt.encode_utf16().collect();
    v.push(0);
    v
}