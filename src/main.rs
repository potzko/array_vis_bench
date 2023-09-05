mod sorts;
mod traits;
fn main() {
    let tmp = traits::log_traits::SortLog::Cmp {
        name: 1,
        ind_a: 2,
        ind_b: 3,
        result: true,
    };
    println!("{:?}", tmp);
    let mut arr = vec![1, 2, 3, 4, 5, 4, 3, 2, 1];
}
