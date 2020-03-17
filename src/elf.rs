use xmas_elf::sections::{SectionHeader, SectionData};
use xmas_elf::ElfFile;
use xmas_elf::program::{FLAG_X, FLAG_R};
use xmas_elf::header;

pub struct ElfLoader<'a> {
    elf: ElfFile<'a>,
}

impl<'a> ElfLoader<'a> {
    pub fn new(input: &'a [u8]) -> Result<ElfLoader<'a>, String> {
        let elf = ElfFile::new(input)?;
        Ok(ElfLoader {
            elf
        })
    }

    fn check_header(&self) -> Result<(), String> {
        //check riscv
        if let header::Machine::Other(id) = self.elf.header.pt2.machine().as_machine() {
            if id == 243 {
                Ok(())
            } else {
                Err(format!("Invalid Arch {:?}!", self.elf.header.pt2.machine()))
            }
        } else {
            Err(format!("Invalid Arch {:?}!", self.elf.header.pt2.machine()))
        }
    }

    fn sections_filter(&self) -> impl Iterator<Item=SectionHeader> {
        self.elf.section_iter().filter(|s| {
            s.flags() & (FLAG_X as u64) | s.flags() & (FLAG_R as u64) != 0
        })
    }

    pub fn load<F: Fn(&str, u64, &[u8]) -> Result<(), String>>(&self, f: F) -> Result<(), String> {
        self.check_header()?;
        let result = self.sections_filter().map(|s| {
            let data = match s.get_data(&self.elf)? {
                SectionData::Undefined(d) => Ok(d),
                _ => Err("Only support Undefined SectionData for now!")
            };
            f(s.get_name(&self.elf)?, s.address(), data?)
        });
        for r in result {
            if let Err(e) = r {
                return Err(e);
            }
        }
        Ok(())
    }
}