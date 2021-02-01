use super::common::generate_malware_ip_list;
use arc_swap::ArcSwap;
use crossbeam_channel;
use crossbeam_utils::thread;
use rand::Rng;
use std::sync::{Arc, RwLock, Mutex};
use std::time::Instant;

pub fn test_arcswap() -> u128 {
    let set1 = Arc::new(generate_malware_ip_list(100_000));
    let config = ArcSwap::from(set1);
    let now = Instant::now();
    let finish_reading = Arc::new(now);
    let elapsed = Arc::new(Mutex::new(0 as u128));
    thread::scope(|scope| {
        scope.spawn(|_| {
            for _ in 0..500 {
                config.store(Arc::new(generate_malware_ip_list(1_000)));
            }
        });
        for _ in 0..3 {
            scope.spawn(|_| {
                let mut rng = rand::thread_rng();
                for _ in 0..1_000_000 {
                    //Supose its a IP extracted from a log
                    let source_ip: u32 = rng.gen();

                    let cfg = config.load();
                    let _contains = (**cfg).contains(&source_ip);
                }
                let mut elapsed_now = elapsed.lock().unwrap();
                (*elapsed_now) = (*finish_reading).elapsed().as_micros();
            });
        }
    })
    .unwrap();
    let elapsed_now = elapsed.lock().unwrap();
    *elapsed_now
    //println!("Test ArcSwap {}", now.elapsed().as_micros());//4009 msecs
}

pub fn test_rwlock() -> u128 {
    let set_lock = Arc::from(RwLock::from(generate_malware_ip_list(100_000)));
    let now = Instant::now();
    let finish_reading = Arc::new(now);
    let elapsed = Arc::new(Mutex::new(0 as u128));
    thread::scope(|scope| {
        scope.spawn(|_| {
            for _ in 0..500 {
                let new_set = generate_malware_ip_list(1_000);
                let mut w = set_lock.write().unwrap();
                (*w) = new_set;
            }
        });
        for _ in 0..3 {
            scope.spawn(|_| {
                let mut rng = rand::thread_rng();
                for _ in 0..1_000_000 {
                    //Supose its a IP extracted from a log
                    let source_ip: u32 = rng.gen();
                    let n = set_lock.read().unwrap();
                    let _contains = (*n).contains(&source_ip);
                }
                let mut elapsed_now = elapsed.lock().unwrap();
                (*elapsed_now) = (*finish_reading).elapsed().as_micros();
            });
        }
    })
    .unwrap();
    let elapsed_now = elapsed.lock().unwrap();
    *elapsed_now
    //println!("Test RWLock {}", now.elapsed().as_micros());//4009 msecs
}

pub fn test_map_in_bottle() -> u128 {
    let (w_ch, r_ch) = crossbeam_channel::bounded(1000);
    let set = Arc::from(generate_malware_ip_list(100_000));
    let now = Instant::now();
    let finish_reading = Arc::new(now);
    let elapsed = Arc::new(Mutex::new(0 as u128));
    thread::scope(|scope| {
        scope.spawn(|_| {
            for _ in 0..500 {
                let new_set = Arc::new(generate_malware_ip_list(1_000));
                let _ = w_ch.send(new_set);
            }
        });
        for _ in 1..4 {
            let recv = r_ch.clone();
            let mut myset = set.clone();
            let elapsed = elapsed.clone();
            let finish_reading = finish_reading.clone();
            scope.spawn(move |_| {
                let mut rng = rand::thread_rng();
                
                match recv.try_recv() {
                    Ok(val) => {
                        myset = val;
                    }
                    Err(_) => {}
                }
                for _ in 0..1_000_000 {
                    //Supose its a IP extracted from a log
                    let source_ip: u32 = rng.gen();
                    let _contains = (*myset).contains(&source_ip);
                }
                let mut elapsed_now = elapsed.lock().unwrap();
                (*elapsed_now) = (*finish_reading).elapsed().as_micros();
            });
        }
    })
    .unwrap();
    let elapsed_now = elapsed.lock().unwrap();
    *elapsed_now
}
