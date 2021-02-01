mod read_only;
mod read_update;
mod read_update_big;
mod read_update_gigant;
mod common;


fn main() {
    println!("READ ONLY");
    println!("t_arcswap t_bottle t_rwlock");
    let mut t_arcswap_sum = 0;
    let mut t_bottle_sum = 0;
    let mut t_rwlock_sum = 0;
    for _ in 0..5 {
        let t_arcswap = read_only::test_arcswap();

        let t_bottle = read_only::test_map_in_bottle();
    
        let t_rwlock = read_only::test_rwlock();

        println!("{} {} {}", t_arcswap,t_bottle,t_rwlock);
        t_arcswap_sum += t_arcswap;
        t_bottle_sum += t_bottle;
        t_rwlock_sum += t_rwlock;
    }
    println!("Avg {} {} {} EPms", 1_000_000_000/t_arcswap_sum,1_000_000_000/t_bottle_sum,1_000_000_000/t_rwlock_sum);

    println!("READ WRITE");
    println!("t_arcswap t_bottle t_rwlock");
    let mut t_arcswap_sum = 0;
    let mut t_bottle_sum = 0;
    let mut t_rwlock_sum = 0;
    for _ in 0..5 {
        let t_arcswap = read_update::test_arcswap();

        let t_bottle = read_update::test_map_in_bottle();
    
        let t_rwlock = read_update::test_rwlock();

        println!("{} {} {}", t_arcswap,t_bottle,t_rwlock);
        t_arcswap_sum += t_arcswap;
        t_bottle_sum += t_bottle;
        t_rwlock_sum += t_rwlock;
    }
    println!("Avg {} {} {} EPms", 1_000_000_000/t_arcswap_sum,1_000_000_000/t_bottle_sum,1_000_000_000/t_rwlock_sum);

    println!("READ WRITE BIG");
    println!("t_arcswap t_bottle t_rwlock");
    let mut t_arcswap_sum = 0;
    let mut t_bottle_sum = 0;
    let mut t_rwlock_sum = 0;
    for _ in 0..5 {
        let t_arcswap = read_update_big::test_arcswap();

        let t_bottle = read_update_big::test_map_in_bottle();
    
        let t_rwlock = read_update_big::test_rwlock();

        println!("{} {} {}", t_arcswap,t_bottle,t_rwlock);
        t_arcswap_sum += t_arcswap;
        t_bottle_sum += t_bottle;
        t_rwlock_sum += t_rwlock;
    }
    println!("Avg {} {} {} EPms", 1_000_000_000/t_arcswap_sum,1_000_000_000/t_bottle_sum,1_000_000_000/t_rwlock_sum);

    println!("READ WRITE GIGANT");
    println!("t_arcswap t_bottle t_rwlock");
    let mut t_arcswap_sum = 0;
    let mut t_bottle_sum = 0;
    let mut t_rwlock_sum = 0;
    for _ in 0..5 {
        let t_arcswap = read_update_gigant::test_arcswap();

        let t_bottle = read_update_gigant::test_map_in_bottle();
    
        let t_rwlock = read_update_gigant::test_rwlock();

        println!("{} {} {}", t_arcswap,t_bottle,t_rwlock);
        t_arcswap_sum += t_arcswap;
        t_bottle_sum += t_bottle;
        t_rwlock_sum += t_rwlock;
    }
    println!("Avg {} {} {} EPms", 1_000_000_000/t_arcswap_sum,1_000_000_000/t_bottle_sum,1_000_000_000/t_rwlock_sum);

}


