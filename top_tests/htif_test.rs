extern crate terminus_spaceport;
extern crate terminus;
use terminus::devices::htif::HTIF;
use terminus_spaceport::memory::region::{U32Access, U64Access};
use terminus_spaceport::EXIT_CTRL;
use terminus_spaceport::devices::term_exit;



fn main() {
    let htif = HTIF::new(0, Some(8), true);
    U64Access::write(&htif, &0, 0x0101_0000_0000_0000u64 | b'x' as u64);
    U64Access::write(&htif, &0, 0x0101_0000_0000_000au64);
    loop {
        if let Ok(msg) = EXIT_CTRL.poll() {
            println!("{}", msg);
            break
        }
        U64Access::write(&htif, &0, 0x0100_0000_0000_0000u64);
        if U32Access::read(&htif, &0x8) & 0xff != 0 {
            println!("get char: {}!", std::char::from_u32(U32Access::read(&htif, &0x8) & 0xff).unwrap());
            U64Access::write(&htif, &0x8, 0);
            U64Access::write(&htif, &0, 1);
        }
        U64Access::write(&htif, &8, 0x0000_0000_0000_0000u64);
    }
    term_exit();
}