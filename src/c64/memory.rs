// memory banks
use std::cell::RefCell;
use std::rc::Rc;
use utils;

pub type MemShared = Rc<RefCell<Memory>>;

pub enum MemType {
    Ram,
    Basic,
    Chargen,
    Io,
    Kernal,
}


// specific memory bank - RAM, ROM, IO
pub struct MemBank {
    bank_type: MemType, // what am I?
    read_only: bool,    // RAM or ROM?
    offset: u16,        // offset from start of address space
    data: Vec<u8>,
}

impl MemBank {
    pub fn new(mem_type: MemType) -> MemBank {
        let mut mem_bank = MemBank {
            bank_type: mem_type,
            read_only: true,
            offset: 0x0000,
            data: Vec::<u8>::new(),
        };

        match mem_bank.bank_type {
            MemType::Basic   => {
                mem_bank.data = utils::open_file("rom/basic.rom", 0);
                mem_bank.offset = 0xA000;
            },
            MemType::Chargen => {
                mem_bank.data = utils::open_file("rom/chargen.rom", 0);
                mem_bank.offset = 0xD000;
            },
            MemType::Kernal  => {
                mem_bank.data = utils::open_file("rom/kernal.rom", 0);
                mem_bank.offset = 0xE000;
            },
            MemType::Ram => {
                mem_bank.data = Vec::<u8>::with_capacity(0x10000);
                for _ in 0..0x10000 {
                    mem_bank.data.push(0);
                }

                mem_bank.read_only = false;
            },
            MemType::Io => {
                mem_bank.data = Vec::<u8>::with_capacity(0x1000);
                for _ in 0..0x1000 {
                    mem_bank.data.push(0);
                }

                mem_bank.offset = 0xD000;
                mem_bank.read_only = false;
            }
        }
        
        mem_bank
    }


    pub fn write(&mut self, addr: u16, val: u8) {
        match self.bank_type {
            MemType::Ram => self.data[(addr - self.offset) as usize] = val,
            MemType::Io => {
                match addr {
                    0xD016          => self.data[(addr - self.offset) as usize] = 0xC0 | val,
                    0xD019          => self.data[(addr - self.offset) as usize] = 0x70 | val,
                    0xD01A          => self.data[(addr - self.offset) as usize] = 0xF0 | val,
                    //0xD01E...0xD01F => (),  // cannot be written on real C64 but allow the VIC to do it anyway
                    0xD020...0xD02E => self.data[(addr - self.offset) as usize] = 0xF0 | val,
                    0xD02F...0xD03F => (),                             // write ignored
                    0xD040...0xD3FF => self.write(0xD000 + (addr % 0x0040), val), // same as 0xD000-0xD03F
                    _ => self.data[(addr - self.offset) as usize] = val
                }
                
            },
            _ => panic!("Can't write to ROM!")
        }
    }


    pub fn read(&mut self, addr: u16) -> u8 {
        match self.bank_type {
            MemType::Io => {
                match addr {
                    0xD016          => 0xC0 | self.data[(addr - self.offset) as usize],
                    0xD018          => 0x01 | self.data[(addr - self.offset) as usize],
                    0xD019          => 0x70 | self.data[(addr - self.offset) as usize],
                    0xD01A          => 0xF0 | self.data[(addr - self.offset) as usize],
                    0xD01E...0xD01F => {                                  // cannot be written, cleared on read
                        let value = self.data[(addr - self.offset) as usize];
                        self.data[(addr - self.offset) as usize] = 0;
                        value
                    },
                    0xD020...0xD02E => 0xF0 | self.data[(addr - self.offset) as usize],
                    0xD02F...0xD03F => 0xFF,                                 // always returns 0xFF
                    0xD040...0xD3FF => self.read(0xD000 + (addr % 0x0040)),  // same as 0xD000-0xD03F
                    _ => self.data[(addr - self.offset) as usize]
                }
            },
            _ => self.data[(addr - self.offset) as usize]
        }
    }    
}


// collective memory storage with all the banks and bank switching support
pub struct Memory {
    ram:     MemBank,
    basic:   MemBank,
    chargen: MemBank,
    io:      MemBank,
    kernal:  MemBank,

    // bank switching flags
    pub exrom:      bool,
    pub game:       bool,
    pub basic_on:   bool,
    pub chargen_on: bool,
    pub io_on:      bool,
    pub kernal_on:  bool,
}

impl Memory {
    pub fn new_shared() -> MemShared {
        Rc::new(RefCell::new(Memory {
            ram:     MemBank::new(MemType::Ram),     // 64k
            basic:   MemBank::new(MemType::Basic),   // 8k
            chargen: MemBank::new(MemType::Chargen), // 4k
            io:      MemBank::new(MemType::Io),      // 4k (VIC, SID, CIA, Color RAM)
            kernal:  MemBank::new(MemType::Kernal),  // 8k
            exrom:      true,
            game:       true,
            basic_on:   false,
            chargen_on: false,
            io_on:      false,
            kernal_on:  false,
        }))
    }
    

    // returns memory bank for current latch setting and address
    pub fn get_bank(&mut self, addr: u16) -> (&mut MemBank) {
        match addr {
            0x0000...0x9FFF => &mut self.ram,
            0xA000...0xBFFF => if self.basic_on { &mut self.basic } else { &mut self.ram },
            0xC000...0xCFFF => &mut self.ram,
            0xD000...0xDFFF => {
                if self.chargen_on { return &mut self.chargen }
                if self.io_on      { return &mut self.io; }
                return &mut self.ram;
            },
            0xE000...0xFFFF => if self.kernal_on  { &mut self.kernal } else { &mut self.ram },
            _ => panic!("Address out of memory range")
        }
    }


    // returns specific modifiable memory bank
    pub fn get_ram_bank(&mut self, bank_type: MemType) -> (&mut MemBank) {
        match bank_type {
            MemType::Ram => &mut self.ram,
            MemType::Io  => &mut self.io,
            _            => panic!("Unrecognized RAM bank"),
        }
    }


    // returns specific non-modifiable memory bank
    pub fn get_rom_bank(&mut self, bank_type: MemType) -> (&mut MemBank) {
        match bank_type {
            MemType::Basic   => &mut self.basic,
            MemType::Chargen => &mut self.chargen,
            MemType::Kernal  => &mut self.kernal,
            _                => panic!("Unrecognized ROM Abank"),
        }
    }    
    

    pub fn reset(&mut self) {
        self.write_byte(0x0000, 0xFF);
        self.write_byte(0x0001, 0x07); // enable kernal, chargen and basic ROMs
    }

    
    // Write a byte to memory - returns whether RAM was written (true) or RAM under ROM (false)
    pub fn write_byte(&mut self, addr: u16, value: u8) -> bool {
        // RAM under ROM written? Return false to let us know about it
        if self.get_bank(addr).read_only {
            self.ram.write(addr, value);
            return false;
        }
        else {
            self.get_bank(addr).write(addr, value);
        }
        
        // update the bank switching flags here, since they can only change on memory write
        // latch byte changed - update bank switching flags
        if addr < 0x0002 {
            self.update_memory_latch();
        }
        
        return true;
    }
    

    // Read a byte from memory
    pub fn read_byte(&mut self, addr: u16) -> u8 {
        // special location: current memory latch settings
        if addr == 0x0001 {
            let ddr = self.ram.read(0x0000);
            let pr  = self.ram.read(0x0001);
            return (ddr & pr) | (!ddr & 0x17);
        }
        
        self.get_bank(addr).read(addr)
    }


    // Read a word from memory (stored in little endian)
    pub fn read_word_le(&mut self, addr: u16) -> u16 {
        let bank = self.get_bank(addr);
        let value_be: u16 = ((bank.read(addr) as u16) << 8 & 0xFF00) |
                            ((bank.read(addr + 0x0001) as u16) & 0x00FF);

        let value_le: u16 = ((value_be << 8) & 0xFF00) | ((value_be >> 8) & 0x00FF);
        value_le
    }


    // *** private functions *** //

    // update status of memory bank latches
    fn update_memory_latch(&mut self) {
        let ddr = self.ram.read(0x0000);
        let pr  = self.ram.read(0x0001);
        let latch = !ddr | pr;

        self.chargen_on = ((latch & 0x04) == 0) && ((latch & 0x03) != 0); // %0xx except %000
        self.io_on      = ((latch & 0x04) != 0) && ((latch & 0x03) != 0); // %1xx except %100
        self.basic_on   = (latch & 0x03) == 3;
        self.kernal_on  = (latch & 0x02) != 0; 
        
        // binary logic is hard
        if self.exrom && !self.game {
            self.basic_on = false;
            self.kernal_on = false;
        }
        if !self.exrom && !self.game {
            self.basic_on = false;
        }
    }
}

