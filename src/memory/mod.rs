#![allow(dead_code)]
pub struct Memory
{
    bytes: [u8;65536]
}

impl Memory
{
    pub fn new() -> Memory
    {
        Memory
        {
            bytes: [0;0x10000] // 64k memory
        }        
    }

    pub fn reset(&mut self)
    {
        self.write_byte(0x0000, 0xFF);
        self.write_byte(0x0001, 0x07);
    }

    // Write a byte to memory
    pub fn write_byte(&mut self, addr: u16, value: u8)
    {
        self.bytes[addr as usize] = value;
    }
    
    // Read a byte from memory
    pub fn read_byte(&mut self, addr: u16) -> u8
    {
        self.bytes[addr as usize]
    }

    // Read a word from memory (stored in little endian)
    pub fn read_word_le(&mut self, addr: u16) -> u16
    {
        let value_be: u16 = ((self.bytes[addr as usize] as u16) << 8 & 0xFF00) |
                            ((self.bytes[(addr + 0x0001) as usize] as u16) & 0x00FF);

        let value_le: u16 = ((value_be << 8) & 0xFF00) | ((value_be >> 8) & 0x00FF);
        value_le
    }

    // Read a word from memory (stored in big endian: swap low<->high)
    pub fn read_word_be(&mut self, addr: u16) -> u16
    {
        let value_le: u16 = ((self.bytes[addr as usize] as u16) << 8 & 0xFF00) |
                            ((self.bytes[(addr + 0x0001) as usize] as u16) & 0x00FF);
        value_le
    }

    // Write word in little endian format (low/high)
    pub fn write_word_le(&mut self, addr: u16, value: u16)
    {
        let value_le_lo: u8 = (((value << 8) & 0xFF00) >> 8 & 0xFF) as u8;
        let value_le_hi: u8 = ((value >> 8) & 0x00FF) as u8;

        self.bytes[addr as usize] = value_le_lo;
        self.bytes[(addr + 0x0001) as usize] = value_le_hi;
    }    

    // Write word in big endian format (high/low)
    pub fn write_word_be(&mut self, addr: u16, value: u16)
    {
        let value_le_lo: u8 = (((value << 8) & 0xFF00) >> 8 & 0xFF) as u8;
        let value_le_hi: u8 = ((value >> 8) & 0x00FF) as u8;
                                                              
        self.bytes[addr as usize] = value_le_hi;
        self.bytes[(addr + 0x0001) as usize] = value_le_lo;
    }    
}

