#![feature(test)]
extern crate test;

use test::Bencher;
use terminus::processor::decode::*;


#[bench]
fn decode_bench(b: &mut Bencher) {
    let code = vec![0x04813823u32, 0x0005c783u32, 0x06010413u32, 0x00093783u32];
    b.iter(|| {
        for c in code.iter() {
            GDECODER.decode(*c).unwrap();
        }
    });
}