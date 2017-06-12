#![feature(proc_macro)]
#![feature(target_feature)]
#![feature(const_fn)]

extern crate runtime_target_feature;

use runtime_target_feature::runtime_target_feature;

#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), runtime_target_feature("+avx"))]
#[cfg_attr(target_arch = "arm", runtime_target_feature("+neon"))]
pub fn sum(input: &[u32]) -> u32 {
    input.iter().sum()
}

#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), runtime_target_feature("+sse4.1"))]
pub fn product(input: &[u32]) -> u32 {
    input.iter().product()
}

#[test]
fn test() {
    let numbers = [1, 2, 3, 4, 5];
    assert_eq!(15, sum(&numbers));
    assert_eq!(120, product(&numbers));
}
