#![allow(dead_code)]
#![allow(unused_imports)]
extern crate xmas_elf;

use std::fs;
use xmas_elf::ElfFile;
use xmas_elf::sections::SectionData::*;
use xmas_elf::symbol_table::*;

#[test]
#[ignore]
fn test_load_elf() {
    let blob = fs::read("top_tests/elf/rv32ui-p-add").expect("Can't read binary");
    let elf = ElfFile::new(blob.as_slice()).expect("Invalid ELF!");
    println!("{:?}", elf.header);
    elf.program_iter().for_each(|p| {
        println!("program:{:?}", p);
        println!("program vaddr :{:x}", p.virtual_addr());
        println!("program flag :{:?}", p.flags());
        println!("size :{}", p.mem_size());
        println!("data:{:?}", p.get_data(&elf));
    });
    elf.section_iter().for_each(|s| {
        let ty = s.get_type().unwrap();
        println!("ty:{:?}", ty);
        println!("flag:{:?}", s.flags());
        println!("addr:{:x}", s.address());
        println!("name:{:?}", s.get_name(&elf));
        println!("data:{:?}", s.get_data(&elf));
        // let name = s.get_name(&elf).unwrap();
        s.get_data(&elf).unwrap();
        // if let SymbolTable64(d) = data {
        //     for e in d {
        //         println!("name:{:?}", e.get_name(&elf));
        //         println!("binding {:?}",e.get_binding());
        //         println!("value {:x}",e.value());
        //         println!("size {}",e.size());
        //         println!("info {}", e.info());
        //         println!("shndx {}", e.shndx());
        //         println!("type {:?}", e.get_type());
        //         println!("type {:?}", e.get_type());
        //     }
        // }
    });
}
