use crate::common::{EvtListenerConfiguration, ListenerConfiguration};
use std::sync::{Arc, Mutex};
use std::fs;
pub static WORKING_DIRECTORY: &str = "C:\\ProgramData\\uLogger";

lazy_static! {
    
    pub static ref LISTENER_LIST: Vec<(&'static str, &'static str)> = {
        let mut m = Vec::new();
        m.push(("Security","*"));
        m.push(("Microsoft-Windows-Sysmon/Operational","*"));
        m
    };
    pub static ref RUNNING_LISTENER_CONFIGURATION : Arc<Mutex<Vec<Arc<Mutex<ListenerConfiguration>>>>> = Arc::new(Mutex::new(Vec::new()));
}

pub fn save_running_configuration() {
    match RUNNING_LISTENER_CONFIGURATION.lock() {
        Ok(guard) => {
            let mut i = 0;
            let mut listeners = vec![];
            for listener in guard.iter() {
                match listener.lock() {
                    Ok(listener_config) => {
                        println!("{:?}",&listener_config);
                        listeners.push(listener_config.clone())
                    }
                    Err(_) => {
                        println!("Cannot aquire mutex {}", i);
                    }
                }
                i += 1;
            }
            let json = serde_json::to_string(&listeners).unwrap();
            save_listeners(&json);
        }
        Err(_) => {
        }
    }
}

pub fn save_listeners(listeners : &str) {
    let working_path = std::path::Path::new(WORKING_DIRECTORY);
    if !working_path.exists() {
        match fs::create_dir(working_path) {
            Ok(()) => {}
            Err(_) => {}
        }
    }
    let listener_path = working_path.join("listeners.json");
    fs::write(&listener_path, listeners);
    
}