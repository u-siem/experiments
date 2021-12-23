use async_std::fs::{File, OpenOptions};
use async_std::fs;
use async_std::prelude::*;

use async_std::io;
use async_std::io::BufReader;
use async_std::io::ErrorKind;
use async_std::io::SeekFrom;
use async_std::path::Path;
use async_std::task::sleep;
use std::time::Duration;

pub struct DnsLogWatcher {
    principal_file :  String,
    reader: BufReader<File>,
    rotation_size : u64,
    force_rotation : u64,
    size : u64,
    pos : u64
}

pub struct LogWatcherConfig {
    size : u64,
    pos : u64
}

impl DnsLogWatcher {

    pub async fn register(log_filename : &str, rotation_size : u64, config : Option<LogWatcherConfig>) -> Result<DnsLogWatcher, io::Error> {
        let f = OpenOptions::new().read(true).write(true).open(&log_filename).await?;
        let metadata = f.metadata().await?;
        let mut reader = BufReader::new(f);
        match config {
            Some(config) => {
                if config.pos < metadata.len() && config.size < metadata.len() {
                    // Bigger file but not rotated
                    reader.seek(SeekFrom::Start(config.pos)).await?;
                    Ok(DnsLogWatcher {
                        principal_file : log_filename.to_string(),
                        reader,
                        rotation_size,
                        force_rotation : std::cmp::max((rotation_size as f64 * 0.9) as u64, std::cmp::max(0i64, rotation_size as i64 - 10000i64) as u64),
                        size : metadata.len(),
                        pos : config.pos
                    })
                }else {
                    reader.seek(SeekFrom::Start(0)).await?;
                    Ok(DnsLogWatcher {
                        principal_file : log_filename.to_string(),
                        reader,
                        rotation_size,
                        force_rotation : std::cmp::max((rotation_size as f64 * 0.9) as u64, std::cmp::max(0i64, rotation_size as i64 - 10000i64) as u64),
                        size : metadata.len(),
                        pos : 0
                    })
                }
            },
            None => {
                reader.seek(SeekFrom::Start(0)).await?;
                Ok(DnsLogWatcher {
                    principal_file : log_filename.to_string(),
                    reader,
                    rotation_size,
                    force_rotation : std::cmp::max((rotation_size as f64 * 0.9) as u64, std::cmp::max(0i64, rotation_size as i64 - 10000i64) as u64),
                    size : metadata.len(),
                    pos : 0
                })
            }
        }
    }

    pub async fn save_config(&mut self) -> Result<LogWatcherConfig, io::Error>
    {
        let file = OpenOptions::new().read(true).write(true).open(&self.principal_file).await?;
        let metadata = file.metadata().await?;
        Ok(LogWatcherConfig {
            pos : self.pos,
            size : metadata.len()
        })
    }

    pub async fn resync_file(&mut self) -> Result<(), io::Error>
    {
        println!("resync_file");
        let file = OpenOptions::new().read(true).write(true).open(&self.principal_file).await?;
        let metadata = file.metadata().await?;
        self.reader = BufReader::new(file);
        self.reader.seek(SeekFrom::Start(self.pos)).await?;
        self.size = metadata.len();
        Ok(())
    }

    pub async fn force_rotation(&mut self) -> Result<(), io::Error>
    {
        println!("Forced rotation");
        let file = OpenOptions::new().read(true).write(true).open(&self.principal_file).await?;
        file.set_len(0).await?;
        self.reader = BufReader::new(file);
        self.reader.seek(SeekFrom::Start(0)).await?;
        self.pos = 0;
        self.size = 0;
        Ok(())
    }

    pub async fn watch(&mut self) -> Result<(), io::Error>{
        loop {
            let mut line = String::new();
            let readed_bytes = self.reader.read_line(&mut line).await?;
            if readed_bytes > 0 {
                self.pos += readed_bytes as u64;
                self.reader.seek(SeekFrom::Start(self.pos)).await?;
                let log_line = line.replace("\n", "");
                if log_line != "" {
                    println!("{}", log_line);
                }
                line.clear();
            }else{
                if self.pos > self.rotation_size {
                    self.force_rotation().await?;
                }
                async_std::task::sleep(Duration::from_millis(5000)).await;
                self.resync_file().await?;
            }
        }
    }
}

#[test]
fn test_log_watch() {
    async_std::task::block_on(async {
        let mut watcher = DnsLogWatcher::register("C:\\dns\\dns.log", 2_000, None).await.unwrap();
        watcher.watch().await.unwrap();
        let config = watcher.save_config().await.unwrap();
    });
    

}