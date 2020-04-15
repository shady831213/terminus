extern crate simple_soft_float;
use simple_soft_float::{F32, FPState, RoundingMode, StatusFlags};
use std::convert::TryFrom;

#[test]
fn soft_float() {
    let mut state = FPState::default();
    println!("state {:?}", state);
    let num = F32::from_bits(0xc49a6333);
    let num2 = F32::from_bits(0x3f8ccccd);
    let num3 = num.add( &num2, Some(RoundingMode::TiesToAway), Some(&mut state));
    println!("{} + {} = {}",f32::from_bits(*num.bits()), f32::from_bits(*num2.bits()), f32::from_bits(*num3.bits()));
    println!("state {:?}", state);
    state.status_flags = StatusFlags::default();
    let num = F32::from_bits(0x40200000);
    let num2 = F32::from_bits(0x3f800000);
    let num3 = num.add( &num2, Some(RoundingMode::TiesToEven), Some(&mut state));
    println!("{} + {} = {}",f32::from_bits(*num.bits()), f32::from_bits(*num2.bits()), f32::from_bits(*num3.bits()));
    println!("state {:?}", state);
}