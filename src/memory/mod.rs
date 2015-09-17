#![allow(dead_code)]
use utils;
pub struct Memory
{
    ram: [u8;0x10000],
    rom: [u8;0x10000],
    // bank switching flags
    basic_on: bool,
    chargen_on: bool,
    io_on: bool,
    kernal_on: bool,
    cart_lo_on: bool, // cart flag - unused for now
    cart_hi_on: bool  // cart flag - unused for now
}

impl Memory
{
    pub fn new() -> Memory
    {
        let mut memory = Memory
        {
            ram: [0;0x10000],   // 64k
            rom: [0;0x10000],   // store ROM data in 64k array so it's easier to address it with bank switching
            basic_on: false,
            chargen_on: false,
            io_on: false,
            kernal_on: false,
            cart_lo_on: false, // unused for now
            cart_hi_on: false, // unused for now
        };

        // load basic
        let mut start_addr: u32 = 0xA000;
        let basic = utils::open_file("rom/basic.rom", 0);
        
        for (i,addr) in (start_addr..0xC000).enumerate()
        {
            memory.rom[addr as usize] = basic[i as usize];
        }

        // load chargen
        start_addr = 0xD000;
        let chargen = utils::open_file("rom/chargen.rom", 0);
        
        for (i,addr) in (start_addr..0xE000).enumerate()
        {
            memory.rom[addr as usize] = chargen[i as usize];
        }
              
        // load kernal
        start_addr = 0xE000;
        let kernal = utils::open_file("rom/kernal.rom", 0);
        
        for (i,addr) in (start_addr..0x10000).enumerate()
        {
            memory.rom[addr as usize] = kernal[i as usize];
        }
        
        memory
    }
    
    // returns memory bank for current address + latch setting and an indicator whether it's read-only to detect ROM writes
    pub fn get_bank(&mut self, addr: u16) -> (&mut [u8], bool)
    {
        match addr
        {
            0x0000...0x9FFF => (&mut self.ram[0..0x10000], false),
            0xA000...0xCFFF => if self.basic_on   { (&mut self.rom[0..0x10000], true) } else { (&mut self.ram[0..0x10000], false) },
            0xD000...0xDFFF => if self.chargen_on { (&mut self.rom[0..0x10000], true) } else { (&mut self.ram[0..0x10000], false) },
            0xE000...0xFFFF => if self.kernal_on  { (&mut self.rom[0..0x10000], true) } else { (&mut self.ram[0..0x10000], false) },
            _ => panic!("Address out of memory range")
        }
    }
    

    pub fn reset(&mut self)
    {
        self.write_byte(0x0000, 0xFF);
        self.write_byte(0x0001, 0x07); // enable kernal, chargen and basic ROMs
    }

    fn update_bank_flags(&mut self)
    {
        // latch state is determined by 3 least significant bits from this location
        let latch = self.ram[0x0001] & 0x07;

        self.chargen_on = ((latch & 0x04) == 0) && ((latch & 0x03) != 0); // %0xx except %000
        self.io_on      = ((latch & 0x04) != 0) && ((latch & 0x03) != 0); // %1xx except %100
        self.basic_on   = (latch & 0x03) == 0x03;
        self.kernal_on  = self.basic_on || ((latch & 0x03) == 0x02);
    }
    
    // Write a byte to memory - returns whether RAM was written (true) or RAM under ROM (false)
    pub fn write_byte(&mut self, addr: u16, value: u8) -> bool
    {
        // parentheses to avoid borrowing issues with changing the flags
        {
            self.ram[addr as usize] = value;
            
            let (_, read_only) = self.get_bank(addr);
            
            // RAM under ROM written? Return false to let us know about it
            if read_only
            {
                return false;
            }
        }

        // update the bank switching flags here, since they can only change on memory write
        // latch byte changed - update bank switching flags
        if addr == 0x0001 { self.update_bank_flags(); }
        
        return true;
    }
    
    // Read a byte from memory
    pub fn read_byte(&mut self, addr: u16) -> u8
    {
        let bank = self.get_bank(addr);
        bank.0[addr as usize]
    }

    // Read a word from memory (stored in little endian)
    pub fn read_word_le(&mut self, addr: u16) -> u16
    {
        let bank = self.get_bank(addr);   
        let value_be: u16 = ((bank.0[addr as usize] as u16) << 8 & 0xFF00) |
                            ((bank.0[(addr + 0x0001) as usize] as u16) & 0x00FF);

        let value_le: u16 = ((value_be << 8) & 0xFF00) | ((value_be >> 8) & 0x00FF);
        value_le
    }

    // Read a word from memory (stored in big endian: swap low<->high)
    pub fn read_word_be(&mut self, addr: u16) -> u16
    {
        let bank = self.get_bank(addr);
        let value_le: u16 = ((bank.0[addr as usize] as u16) << 8 & 0xFF00) |
                            ((bank.0[(addr + 0x0001) as usize] as u16) & 0x00FF);
        value_le
    }

    // Write word in little endian format (low/high)
    pub fn write_word_le(&mut self, addr: u16, value: u16) -> bool
    {
        let value_le_lo: u8 = (((value << 8) & 0xFF00) >> 8 & 0xFF) as u8;
        let value_le_hi: u8 = ((value >> 8) & 0x00FF) as u8;

        let hi = self.write_byte(addr, value_le_lo);
        let lo = self.write_byte(addr + 0x0001, value_le_hi);

        return hi && lo;
    }

    // Write word in big endian format (high/low)
    pub fn write_word_be(&mut self, addr: u16, value: u16) -> bool
    {
        let value_le_lo: u8 = (((value << 8) & 0xFF00) >> 8 & 0xFF) as u8;
        let value_le_hi: u8 = ((value >> 8) & 0x00FF) as u8;

        let hi = self.write_byte(addr, value_le_hi);
        let lo = self.write_byte(addr + 0x0001, value_le_lo);

        return hi && lo;
    }

    // Debug: overwrite a byte in ROM
    pub fn debug_write_rom(&mut self, addr: u16, value: u8)
    {
        self.rom[addr as usize] = value;
    }

}

