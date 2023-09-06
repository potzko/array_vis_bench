use rand::Rng;

pub fn get_rand_arr(length: usize) -> Vec<u64> {
    let mut rng = rand::thread_rng();
    let mut vec = Vec::with_capacity(length);
    for _ in 0..length {
        vec.push(rng.gen());
    }
    vec
}

pub fn get_rand_arr_in_range(length: usize, min: usize, max: usize) -> Vec<u64> {
    let mut rng = rand::thread_rng();
    let mut vec = Vec::with_capacity(length);
    for _ in 0..length {
        vec.push(rng.gen_range(min..max) as u64);
    }
    vec
}
