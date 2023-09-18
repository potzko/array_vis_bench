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

pub fn get_arr(length: usize) -> Vec<u64> {
    let mut ret: Vec<u64> = Vec::with_capacity(length);
    for i in 0..length {
        ret.push(i as u64);
    }
    ret
}

pub fn reverse_arr<T>(arr: &mut [T], start: usize, end: usize) {
    let len = end - start;
    for i in 0..len / 2 {
        arr.swap(i, len - i - 1);
    }
}

pub fn get_reversed_arr(length: usize) -> Vec<u64> {
    let mut ret = get_arr(length);
    reverse_arr(&mut ret, 0, length);
    ret
}
