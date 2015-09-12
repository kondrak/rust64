#![allow(dead_code)]
use utils;
pub struct Memory
{
    ram: [u8;0x10000],
    rom: [u8;0x10000],
    // bank switching flags
    basic_on: bool,
    chargen_on: bool,
    kernal_on: bool,
}

impl Memory
{
    pub fn new() -> Memory
    {
        let mut memory = Memory
        {
            ram: [0;0x10000],   // 64k
            rom: [0;0x10000],   // store ROM data in 64k array so it's easier to address it with bank switching
            basic_on: true,
            chargen_on: true,
            kernal_on: true
        };

        // load basic
        let mut start_addr: u32 = 0xA000;
        let basic = utils::open_file("rom/basic.rom");
        
        for (i,addr) in (start_addr..0xC000).enumerate()
        {
            memory.rom[addr as usize] = basic[i as usize];
        }

        // load chargen
        start_addr = 0xD000;
        let chargen = utils::open_file("rom/chargen.rom");
        
        for (i,addr) in (start_addr..0xE000).enumerate()
        {
            memory.rom[addr as usize] = chargen[i as usize];
        }
              
        // load kernal
        start_addr = 0xE000;
        let kernal = utils::open_file("rom/kernal.rom");
        
        for (i,addr) in (start_addr..0x10000).enumerate()
        {
            memory.rom[addr as usize] = kernal[i as usize];
        }
        
        memory
    }

    pub fn get_bank(&mut self, addr: u16) -> &mut [u8]
    {
        match addr
        {
            0x0000...0x9FFF => &mut self.ram[0..0x10000],
            0xA000...0xCFFF => if self.basic_on   { &mut self.rom[0..0x10000] } else { &mut self.ram[0..0x10000] },
            0xD000...0xDFFF => if self.chargen_on { &mut self.rom[0..0x10000] } else { &mut self.ram[0..0x10000] },
            0xE000...0xFFFF => if self.kernal_on  { &mut self.rom[0..0x10000] } else { &mut self.ram[0..0x10000] },
            _ => panic!("Address out of memory range")
        }
    }
    

    pub fn reset(&mut self)
    {
        self.write_byte(0x0000, 0xFF);
        self.write_byte(0x0001, 0x07); // enable kernal, chargen and basic access

        self.update_bank_flags();
    }

    fn update_bank_flags(&mut self)
    {
        let latch = self.ram[0x0001];
        self.basic_on   = (latch & 0x0B) != 0 || (latch & 0x0F) != 0;
        self.chargen_on = (latch & 0x09) != 0 || (latch & 0x0A) != 0 || (latch & 0x0B) != 0 || (latch == 0x02) || (latch == 0x03);
        self.kernal_on  = (latch & 0x0F) != 0 || (latch & 0x0E) != 0 || (latch & 0x0B) != 0 || (latch &0x0A) != 0 || (latch == 0x02) || (latch == 0x03) || (latch == 0x07) || (latch == 0x06);
    }
    
    // Write a byte to memory
    pub fn write_byte(&mut self, addr: u16, value: u8)
    {
        // parentheses to avoid borrowing issues with changing the flags
        {
            let bank = self.get_bank(addr);
            bank[addr as usize] = value;
        }

        // update the bank switching flags here, since they can only change on memory write
        // latch byte changed - update bank switching flags
        if addr == 0x0001 { self.update_bank_flags(); }
    }
    
    // Read a byte from memory
    pub fn read_byte(&mut self, addr: u16) -> u8
    {
        let bank = self.get_bank(addr);
        bank[addr as usize]
    }

    // Read a word from memory (stored in little endian)
    pub fn read_word_le(&mut self, addr: u16) -> u16
    {
        let bank = self.get_bank(addr);   
        let value_be: u16 = ((bank[addr as usize] as u16) << 8 & 0xFF00) |
                            ((bank[(addr + 0x0001) as usize] as u16) & 0x00FF);

        let value_le: u16 = ((value_be << 8) & 0xFF00) | ((value_be >> 8) & 0x00FF);
        value_le
    }

    // Read a word from memory (stored in big endian: swap low<->high)
    pub fn read_word_be(&mut self, addr: u16) -> u16
    {
        let bank = self.get_bank(addr);
        let value_le: u16 = ((bank[addr as usize] as u16) << 8 & 0xFF00) |
                            ((bank[(addr + 0x0001) as usize] as u16) & 0x00FF);
        value_le
    }

    // Write word in little endian format (low/high)
    pub fn write_word_le(&mut self, addr: u16, value: u16)
    {
        let bank = self.get_bank(addr);
        let value_le_lo: u8 = (((value << 8) & 0xFF00) >> 8 & 0xFF) as u8;
        let value_le_hi: u8 = ((value >> 8) & 0x00FF) as u8;

        bank[addr as usize] = value_le_lo;
        bank[(addr + 0x0001) as usize] = value_le_hi;
    }

    // Write word in big endian format (high/low)
    pub fn write_word_be(&mut self, addr: u16, value: u16)
    {
        let bank = self.get_bank(addr);
        let value_le_lo: u8 = (((value << 8) & 0xFF00) >> 8 & 0xFF) as u8;
        let value_le_hi: u8 = ((value >> 8) & 0x00FF) as u8;

        bank[addr as usize] = value_le_hi;
        bank[(addr + 0x0001) as usize] = value_le_lo;
    }    
}

