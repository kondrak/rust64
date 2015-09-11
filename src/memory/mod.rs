#![allow(dead_code)]
use utils;
pub struct Memory
{
    // RAM
    ram: [u8;65536],
    // ROM
    kernal: [u8;8192],
    basic:  [u8;8192],
    chargen: [u8;4096]
}

impl Memory
{
    pub fn new() -> Memory
    {
        let mut memory = Memory
        {
            ram: [0;0x10000],   // 64k
            kernal: [0;0x2000], // 8k
            basic: [0;0x2000],  // 8k
            chargen: [0;0x1000] // 4k
        };

        // load kernal
        let kernal = utils::open_file("rom/kernal.rom");
        for i in (0..0x2000)
        {
            memory.kernal[i] = kernal[i];
        }

        // load basic
        let basic = utils::open_file("rom/basic.rom");    
        for i in (0..0x2000)
        {
            memory.basic[i] = basic[i];
        }

        // load chargen
        let chargen = utils::open_file("rom/chargen.rom");    
        for i in (0..0x1000)
        {
            memory.chargen[i] = chargen[i];
        }  

        memory
    }

    pub fn reset(&mut self)
    {
        self.write_byte(0x0000, 0xFF);
        self.write_byte(0x0001, 0x07); // enable kernal, chargen and basic access
    }

    // Write a byte to memory
    pub fn write_byte(&mut self, addr: u16, value: u8)
    {
        self.ram[addr as usize] = value;
    }
    
    // Read a byte from memory
    pub fn read_byte(&mut self, addr: u16) -> u8
    {
        self.ram[addr as usize]
    }

    // Read a word from memory (stored in little endian)
    pub fn read_word_le(&mut self, addr: u16) -> u16
    {
        let value_be: u16 = ((self.ram[addr as usize] as u16) << 8 & 0xFF00) |
                            ((self.ram[(addr + 0x0001) as usize] as u16) & 0x00FF);

        let value_le: u16 = ((value_be << 8) & 0xFF00) | ((value_be >> 8) & 0x00FF);
        value_le
    }

    // Read a word from memory (stored in big endian: swap low<->high)
    pub fn read_word_be(&mut self, addr: u16) -> u16
    {
        let value_le: u16 = ((self.ram[addr as usize] as u16) << 8 & 0xFF00) |
                            ((self.ram[(addr + 0x0001) as usize] as u16) & 0x00FF);
        value_le
    }

    // Write word in little endian format (low/high)
    pub fn write_word_le(&mut self, addr: u16, value: u16)
    {
        let value_le_lo: u8 = (((value << 8) & 0xFF00) >> 8 & 0xFF) as u8;
        let value_le_hi: u8 = ((value >> 8) & 0x00FF) as u8;

        self.ram[addr as usize] = value_le_lo;
        self.ram[(addr + 0x0001) as usize] = value_le_hi;
    }    

    // Write word in big endian format (high/low)
    pub fn write_word_be(&mut self, addr: u16, value: u16)
    {
        let value_le_lo: u8 = (((value << 8) & 0xFF00) >> 8 & 0xFF) as u8;
        let value_le_hi: u8 = ((value >> 8) & 0x00FF) as u8;
                                                              
        self.ram[addr as usize] = value_le_hi;
        self.ram[(addr + 0x0001) as usize] = value_le_lo;
    }    
}

