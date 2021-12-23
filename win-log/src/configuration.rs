use crate::common::ListenerConfiguration;
use std::sync::{Arc, Mutex};

pub static WORKING_DIRECTORY: &str = "C:\\ProgramData\\uLogger";

lazy_static! {
    
    pub static ref LISTENER_LIST: Vec<(&'static str, &'static str)> = {
        let mut m = Vec::new();
        m.push(("Security","*"));
        m.push(("Microsoft-Windows-Sysmon/Operational","*"));
        m
    };
    pub static ref RUNNING_LISTENER_CONFIGURATION : Arc<Mutex<Vec<ListenerConfiguration>>> = Arc::new(Mutex::new(Vec::new()));
}
