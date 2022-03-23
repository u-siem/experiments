use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use async_std::channel::{Receiver, Sender};
use crate::common::{ListenerConfiguration, ULoggerError, EvtListenerConfiguration};

use self::winevt::WinEvtListener;

pub mod dns;
pub mod dns_file_watcher;
pub mod winevt;

pub struct ListenerController {
    config : Arc<Mutex<Vec<Arc<Mutex<ListenerConfiguration>>>>>,
    working_directory : String,
    win_evt_listeners : Vec<WinEvtListener>,
    file_listeners : Vec<()>,
    log_sender: Sender<String>

}

impl ListenerController {

    pub fn init(working_directory : String, log_sender: Sender<String>) -> Result<Self, ULoggerError> {
        let listener_configurations = Self::get_listeners_or_default(&working_directory)?;
        let mut configurations = Vec::new();
        let mut winevt  = WinEvtListener::init();
        for config in listener_configurations {
            let listener_conf = if let ListenerConfiguration::Event(_) = config {
                let listener_conf = Arc::new(Mutex::new(config));
                winevt.add_configuration(listener_conf.clone());
                listener_conf
            }else{
                Arc::new(Mutex::new(config))
            };
            configurations.push(listener_conf);
        }
        let config : Arc<Mutex<Vec<Arc<Mutex<ListenerConfiguration>>>>> = Arc::new(Mutex::new(configurations));
        
        Ok(Self {
            working_directory,
            win_evt_listeners : Vec::new(),
            file_listeners : Vec::new(),
            log_sender,
            config
        })
    }

    fn default_listeners() -> Vec<(&'static str, &'static str)> {
        let mut m = Vec::new();
        m.push(("Security","*"));
        m.push(("Microsoft-Windows-Sysmon/Operational","*"));
        m
    }

    fn get_listeners_or_default(working_directory : &str) -> Result<Vec<ListenerConfiguration>, ULoggerError> {
        match get_listeners(working_directory) {
            Ok(listeners) => Ok(listeners),
            Err(_) => {
                let mut listener_list = Vec::new();
                for listener in Self::default_listeners().iter() {
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
    

    pub fn start_listeners(&self) -> Result<Vec<async_std::task::JoinHandle<()>>, ULoggerError> {
        let mut handlers = Vec::with_capacity(128);
        for win_listener in &self.win_evt_listeners {
            let handles = match win_listener.start_listeners(self.log_sender.clone()) {
                Ok(handle) => handle,
                Err(e) => {
                    return Err(e)
                }
            };
            for handle in handles {
                handlers.push(handle);
            }
        }
        Ok(handlers)
    }

    
    
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

