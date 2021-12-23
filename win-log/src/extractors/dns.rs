use super::super::common::{buffer_to_utf8_string, to_pwstr, read_registry_sz, read_registry_dword};
use windows::Win32::System::Registry;

pub struct DnsConfiguration {
    pub log_file: String,
    pub log_file_max_size: u32,
    pub log_level: DnsLogLevel,
    pub event_log_level: EventLogLevel,
}

pub enum EventLogLevel {
    NoEvents,
    Errors,
    ErrorsAndWarnings,
    All,
}

pub struct DnsLogLevel {
    pub no_match: bool,
    pub details: bool,
    pub tcp: bool,
    pub udp: bool,
    pub inbound: bool,
    pub outbound: bool,
    pub answer: bool,
    pub updates: bool,
    pub requests: bool,
    pub notifications: bool,
    pub querys: bool,
}

impl DnsLogLevel {
    pub fn new(flag : u32) -> Self {
        Self {
            no_match : (flag & (1 << 25)) > 0,
            details : (flag & (1 << 24)) > 0,
            tcp : (flag & (1 << 15)) > 0,
            udp : (flag & (1 << 14)) > 0,
            inbound : (flag & (1 << 13)) > 0,
            outbound : (flag & (1 << 12)) > 0,
            answer : (flag & (1 << 9)) > 0,
            requests : (flag & (1 << 8)) > 0,
            updates : (flag & (1 << 5)) > 0,
            notifications : (flag & (1 << 4)) > 0,
            querys : (flag & 1) > 0,
        }
    }

    pub fn as_flag(&self) -> u32 {
        let mut to_ret = 0;
        if self.no_match{
            to_ret += 1<< 25
        }
        if self.details{
            to_ret += 1<< 24
        }
        if self.tcp{
            to_ret += 1<< 15
        }
        if self.udp{
            to_ret += 1<< 14
        }
        if self.inbound{
            to_ret += 1<< 13
        }
        if self.outbound{
            to_ret += 1<< 12
        }
        if self.answer{
            to_ret += 1<< 9
        }
        if self.requests{
            to_ret += 1<< 8
        }
        if self.updates{
            to_ret += 1<< 5
        }
        if self.notifications{
            to_ret += 1<< 4
        }
        if self.querys{
            to_ret += 1
        }
        to_ret
    }
}

pub unsafe fn read_dns_configuration() -> Option<DnsConfiguration> {
    let log_file_path = match read_registry_sz(Registry::HKEY_LOCAL_MACHINE, "SYSTEM\\CurrentControlSet\\Services\\DNS\\Parameters", "LogFilePath") {
        Ok(txt) => txt,
        Err(_) => String::from("C:\\Windows\\System32\\dns\\dns.log")
    };
    let log_file_max_size = match read_registry_dword(Registry::HKEY_LOCAL_MACHINE, "SYSTEM\\CurrentControlSet\\Services\\DNS\\Parameters", "LogFileMaxSize") {
        Ok(txt) => txt,
        Err(_) => 0
    };
    let log_level = match read_registry_dword(Registry::HKEY_LOCAL_MACHINE, "SYSTEM\\CurrentControlSet\\Services\\DNS\\Parameters", "LogLevel") {
        Ok(txt) => txt,
        Err(_) => 0
    };
    let event_log_level = match read_registry_dword(Registry::HKEY_LOCAL_MACHINE, "SYSTEM\\CurrentControlSet\\Services\\DNS\\Parameters", "EventLogLevel") {
        Ok(txt) => txt,
        Err(_) => 0
    };
    let event_log_level = match event_log_level {
        0 => EventLogLevel::NoEvents,
        1 => EventLogLevel::Errors,
        2 => EventLogLevel::ErrorsAndWarnings,
        7 => EventLogLevel::All,
        _ => EventLogLevel::NoEvents
    };
    if log_level != 0 {
        return Some(DnsConfiguration {
            log_file : log_file_path,
            log_file_max_size,
            log_level : DnsLogLevel::new(log_level),
            event_log_level
        })
    }
    None
}

#[test]
fn check_dns_config() {
    unsafe {
        let config = read_dns_configuration().expect("Must retrieve DNS config");
        assert_eq!(config.log_file, "C:\\dns\\dns.log");
        assert_eq!(config.log_file_max_size, 100000000);
        assert_eq!(config.log_level.as_flag(), 50393905);
    }
}


#[test]
fn test_log_level() {
    // 20737 = outbound, udp, querys, requests
    let log_level = DnsLogLevel::new(20737);
    assert!(log_level.outbound);
    assert!(!log_level.inbound);
    assert!(log_level.querys);
    assert!(log_level.requests);
    assert!(log_level.udp);
    // 50393905 = ALL true
    let log_level = DnsLogLevel::new(50393905);
    assert!(log_level.outbound);
    assert!(log_level.inbound);
    assert!(log_level.querys);
    assert!(log_level.requests);
    assert!(log_level.notifications);
    assert!(log_level.no_match);
    assert!(log_level.updates);
    assert!(log_level.udp);
    assert!(log_level.tcp);
    assert_eq!(log_level.as_flag(), 50393905);
}
