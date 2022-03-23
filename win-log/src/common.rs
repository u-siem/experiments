use std::sync::atomic::AtomicBool;

use serde::{Serialize, Deserialize};
use windows::Win32::System::Registry;
use windows::Win32::Foundation;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventBookmark {
    pub xml : String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EvtListenerConfiguration {
    pub bookmark : Option<EventBookmark>,
    pub query : String,
    pub channel : String,
    #[serde(skip_serializing, default = "default_stopper")]
    pub stop : Arc<AtomicBool>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileListenerConfiguration {
    /// Position in the file
    pub position : usize,
    /// Size of the file as to detect rotations
    pub size : usize,
    #[serde(skip_serializing, default = "default_stopper")]
    pub stop : Arc<AtomicBool>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ListenerConfiguration {
    Event(EvtListenerConfiguration),
    File(FileListenerConfiguration)

}
impl Clone for EvtListenerConfiguration {
    fn clone(&self) -> Self {
        EvtListenerConfiguration {
            bookmark : self.bookmark.clone(),
            query : self.query.to_string(),
            channel : self.channel.to_string(),
            stop : Arc::new(AtomicBool::new(false))
        }
    }
}
fn default_stopper() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}
impl EvtListenerConfiguration {
    pub fn get_stop_listener(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stop)
    }
}

#[derive(Debug)]
pub enum ULoggerError {
    InvalidPath(String),
    InvalidConfigFile(String),
    ErrorProcessingEvent(String),
    EventChannelNotFound(String),
    EventQueryInvalid(String),
    InvalidBookmark(String)
}
#[derive(Debug)]
pub enum RegistryError {
    NotExists,
    InvalidPath,
    InvalidHkey,
    InvalidSubKey,
    InvalidValue,
    OpenKeyError(u32),
    QueryValueError(u32),
    CloseValueError(u32),
    InvalidValueType(u32)
}

pub fn to_pwstr(val : &str) -> Vec<u16> {
    let mut val = val.encode_utf16().collect::<Vec<u16>>();
    val.push(0);
    val
}

pub unsafe fn buffer_to_utf16_string(buffer : &mut Vec<u8>, bufferused : usize) -> String {
    let slice = std::slice::from_raw_parts(buffer.as_mut_ptr() as _, (bufferused/2) as usize);
    String::from_utf16_lossy(slice)
}

pub unsafe fn buffer_to_utf8_string(buffer : &mut Vec<u8>, bufferused : usize) -> String {
    let bufferused = if bufferused > 0 && buffer[bufferused - 1] == 0 {bufferused - 1} else {bufferused};
    let slice = std::slice::from_raw_parts(buffer.as_mut_ptr() as _, bufferused as usize);
    String::from_utf8_lossy(slice).to_string()
}
pub unsafe fn buffer_to_utf8_string_slice(buffer : &[u8], bufferused : usize) -> String {
    let bufferused = if bufferused > 0 && buffer[bufferused - 1] == 0 {bufferused - 1} else {bufferused};
    String::from_utf8_lossy(&buffer[0..bufferused]).to_string()
}

pub unsafe fn read_registry_sz(hkey : Registry::HKEY, subkey : &str, value_name : &str) -> Result<String,RegistryError> {
    let mut parameter_key: Registry::HKEY = hkey;
    let mut size = 2048;
    let mut returned_text : [u8; 2048] = [0; 2048];
    
    let lpreserved = std::ptr::null_mut();
    let mut value_type : Registry::REG_VALUE_TYPE = Registry::REG_SZ;

    let error = Registry::RegOpenKeyA(
        hkey,
        subkey,
        &mut parameter_key,
    );
    if error != 0 {
        return Err(RegistryError::OpenKeyError(error))
    }

    let error = Registry::RegQueryValueExA(
        parameter_key,
        value_name,
        lpreserved,
        &mut value_type,
        returned_text.as_mut_ptr(),
        &mut size,
    );
    if error != 0 {
        return Err(RegistryError::QueryValueError(error))
    }
    let error = Registry::RegCloseKey(parameter_key);
    if error != 0 {
        return Err(RegistryError::CloseValueError(error))
    }
    if value_type != Registry::REG_SZ {
        return Err(RegistryError::InvalidValueType(value_type))
    }
    if size == 0{
        return Ok(String::default())
    }
    Ok(buffer_to_utf8_string_slice(&mut returned_text, size as usize))
}

pub unsafe fn read_registry_dword(hkey : Registry::HKEY, subkey : &str, value_name : &str) -> Result<u32,RegistryError> {
    let mut parameter_key: Registry::HKEY = hkey;
    let mut size = 2048;
    let mut returned_u32 : [u8; 4] = [0; 4];
    
    let lpreserved = std::ptr::null_mut();
    let mut value_type : Registry::REG_VALUE_TYPE = Registry::REG_DWORD;

    let error = Registry::RegOpenKeyA(
        hkey,
        subkey,
        &mut parameter_key,
    );
    if error != 0 {
        return Err(RegistryError::OpenKeyError(error))
    }

    let error = Registry::RegQueryValueExA(
        parameter_key,
        value_name,
        lpreserved,
        &mut value_type,
        returned_u32.as_mut_ptr(),
        &mut size,
    );
    if value_type != Registry::REG_DWORD {
        return Err(RegistryError::InvalidValueType(value_type))
    }
    if error != 0 {
        return Err(RegistryError::QueryValueError(error))
    }
    let error = Registry::RegCloseKey(parameter_key);
    if error != 0 {
        return Err(RegistryError::CloseValueError(error))
    }
    Ok(u32::from_le_bytes(returned_u32))
}