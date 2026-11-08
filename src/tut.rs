#![allow(unused)]
use std::io;

fn main() {
    let e = loh(123);
    println!("{}", e);
}

fn loh(n: i32) -> u32 {
    let str: String = n.to_string();
    let char_vec: Vec<char> = str.chars().collect();
    let mut sum = 0;
    for ch in char_vec {
        sum += ch.to_digit(10).unwrap();
    }
    sum
}
