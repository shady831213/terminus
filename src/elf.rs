use xmas_elf::ElfFile;
use xmas_elf::program::SegmentData;
use xmas_elf::header;
use xmas_elf::sections::SectionHeader;

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

    pub fn htif_section(&self) -> Option<SectionHeader> {
        if let Some(s) = self.elf.find_section_by_name(".tohost") {
            Some(s)
        } else if let Some(s) = self.elf.find_section_by_name(".htif") {
            Some(s)
        } else {
            None
        }
    }

    pub fn entry_point(&self) -> u64 {
        self.elf.header.pt2.entry_point()
    }

    pub fn load<F: Fn(u64, &[u8]) -> Result<(), String>>(&self, f: F) -> Result<(), String> {
        self.check_header()?;
        let result = self.elf.program_iter().map(|p| {
            let data = match p.get_data(&self.elf)? {
                SegmentData::Undefined(d) => Ok(d),
                _ => Err("Only support Undefined SectionData for now!")
            };
            f(p.virtual_addr(), data?)
        });
        for r in result {
            if let Err(e) = r {
                return Err(e);
            }
        }
        Ok(())
    }
}