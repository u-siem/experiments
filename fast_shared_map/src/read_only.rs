use super::common::generate_malware_ip_list;
use arc_swap::ArcSwap;
use crossbeam_channel;
use crossbeam_utils::thread;
use rand::Rng;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub fn test_arcswap() -> u128 {
    let set1 = Arc::new(generate_malware_ip_list(100_000));
    let config = ArcSwap::from(set1);
    let now = Instant::now();
    thread::scope(|scope| {
        for _ in 0..3 {
            scope.spawn(|_| {
                let mut rng = rand::thread_rng();
                for _ in 0..1_000_000 {
                    //Supose its a IP extracted from a log
                    let source_ip: u32 = rng.gen();

                    let cfg = config.load();
                    let _contains = (**cfg).contains(&source_ip);
                }
            });
        }
    })
    .unwrap();
    now.elapsed().as_micros()
    //println!("Test ArcSwap {}", now.elapsed().as_micros());//4009 msecs
}

pub fn test_rwlock() -> u128 {
    let set_lock = Arc::from(RwLock::from(generate_malware_ip_list(100_000)));
    let now = Instant::now();
    thread::scope(|scope| {
        for _ in 0..3 {
            scope.spawn(|_| {
                let mut rng = rand::thread_rng();
                for _ in 0..1_000_000 {
                    //Supose its a IP extracted from a log
                    let source_ip: u32 = rng.gen();
                    let n = set_lock.read().unwrap();
                    let _contains = (*n).contains(&source_ip);
                }
            });
        }
    })
    .unwrap();
    now.elapsed().as_micros()
    //println!("Test RWLock {}", now.elapsed().as_micros());//4009 msecs
}

pub fn test_map_in_bottle() -> u128 {
    let (_w_ch, r_ch) = crossbeam_channel::bounded(100);
    let set = Arc::from(generate_malware_ip_list(100_000));
    let now = Instant::now();
    thread::scope(|scope| {
        for _ in 0..3 {
            let recv = r_ch.clone();
            let mut myset = set.clone();
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
            });
        }
    })
    .unwrap();
    now.elapsed().as_micros()
}
