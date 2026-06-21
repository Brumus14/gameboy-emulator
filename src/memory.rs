#[derive(Debug)]
enum Region {
    RomBank0(u16),
    RomBankN(u16),
    VideoMemory(u16),
    ExternalMemory(u16),
    WorkMemory(u16),
    EchoArea(u16),
    ObjectAttributeMemory(u16),
    Unused,
    IoRegisters(u16),
    HighMemory(u16),
    InterruptEnableRegister,
}

pub struct Memory {
    work_memory: [u8; 8192],
    video_memory: [u8; 8192],
    object_attribute_memory: [u8; 160],
    high_memory: [u8; 127],
    interrupt_enable_register: u8,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            work_memory: [0; 8192],
            video_memory: [0; 8192],
            object_attribute_memory: [0; 160],
            high_memory: [0; 127],
            interrupt_enable_register: 0,
        }
    }

    fn decode_address(address: u16) -> Region {
        match address {
            0x0000..0x4000 => !todo!("rom bank 00"),
            0x4000..0x8000 => !todo!("rom bank NN"),
            0x8000..0xA000 => Region::VideoMemory(address - 0x8000),
            0xA000..0xC000 => Region::ExternalMemory(address - 0xA000),
            0xC000..0xE000 => Region::WorkMemory(address - 0xC000),
            0xE000..0xFE00 => Region::EchoArea(address - 0xE000),
            0xFE00..0xFEA0 => Region::ObjectAttributeMemory(address - 0xFE00),
            0xFEA0..0xFF00 => Region::Unused,
            0xFF00..0xFF80 => !todo!("i/o registers"),
            0xFF80..0xFFFF => Region::HighMemory(address - 0xFF80),
            0xFFFF => Region::InterruptEnableRegister,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match Self::decode_address(address) {
            Region::RomBank0(offset) => 0,
            Region::RomBankN(offset) => 0,
            Region::VideoMemory(offset) => self.video_memory[offset as usize],
            Region::ExternalMemory(offset) => 0,
            Region::WorkMemory(offset) => self.work_memory[offset as usize],
            Region::EchoArea(offset) => self.work_memory[(offset + 0xC000) as usize],
            Region::ObjectAttributeMemory(offset) => self.object_attribute_memory[offset as usize],
            Region::Unused => 0,
            Region::IoRegisters(offset) => 0,
            Region::HighMemory(offset) => self.high_memory[offset as usize],
            Region::InterruptEnableRegister => self.interrupt_enable_register,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        *match Self::decode_address(address) {
            Region::RomBank0(offset) => !todo!(),
            Region::RomBankN(offset) => !todo!(),
            Region::VideoMemory(offset) => &mut self.video_memory[offset as usize],
            Region::ExternalMemory(offset) => !todo!(),
            Region::WorkMemory(offset) => &mut self.work_memory[offset as usize],
            Region::EchoArea(offset) => &mut self.work_memory[(offset + 0xC000) as usize],
            Region::ObjectAttributeMemory(offset) => {
                &mut self.object_attribute_memory[offset as usize]
            }
            Region::Unused => !todo!(),
            Region::IoRegisters(offset) => !todo!(),
            Region::HighMemory(offset) => &mut self.high_memory[offset as usize],
            Region::InterruptEnableRegister => &mut self.interrupt_enable_register,
        } = value;
    }
}
