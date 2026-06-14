#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{get_time, yield_};

#[unsafe(no_mangle)]
fn main() -> i32 {
    let start = get_time();
    for i in 0..5 {
        println!("My test iteration {}", i);
        yield_();
    }
    let end = get_time();
    println!("Test done! Time elapsed: {} ms", end - start);
    0
}
