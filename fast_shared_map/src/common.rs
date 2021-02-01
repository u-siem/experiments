use std::collections::BTreeSet;

pub fn generate_malware_ip_list(size : u32) -> BTreeSet<u32> {
    //println!("Generating dataset...");
    let mut ret = BTreeSet::new();
    for i in 0..size {
        ret.insert(i);
    }
    ret
}