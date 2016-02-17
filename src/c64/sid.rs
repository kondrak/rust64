// SID
extern crate rand;
use c64::memory;
use c64::cpu;
use std::cell::RefCell;
use std::rc::Rc;

pub type SIDShared = Rc<RefCell<SID>>;

pub struct SID
{
    mem_ref: Option<memory::MemShared>,
    last_sid_byte: u8,
}

impl SID
{
    pub fn new_shared() -> SIDShared
    {
        Rc::new(RefCell::new(SID
        {
            mem_ref: None,
            last_sid_byte: 0,
        }))
    }

    pub fn set_references(&mut self, memref: memory::MemShared)
    {
        self.mem_ref = Some(memref);
    }

    pub fn read_register(&mut self, addr: u16) -> u8
    {
        // most SID registers are write-only. The write to IO RAM is performed
        // so that the debugger can print out the value fetched by the CPU
        match addr
        {
            0xD419...0xD41A => {
                self.last_sid_byte = 0;
                let rval = 0xFF;
                as_ref!(self.mem_ref).get_ram_bank(memory::MemType::IO).write(addr, rval);
                rval
            },
            0xD41B...0xD41C => {
                self.last_sid_byte = 0;
                let rval = rand::random::<u8>();
                as_ref!(self.mem_ref).get_ram_bank(memory::MemType::IO).write(addr, rval);
                rval
            },
            0xD420...0xD7FF => self.read_register(0xD400 + (addr % 0x0020)),
            _               =>  {
                let rval = self.last_sid_byte;
                as_ref!(self.mem_ref).get_ram_bank(memory::MemType::IO).write(addr, rval);
                rval
            }
        }
    }
    
    pub fn write_register(&mut self, addr: u16, value: u8, on_sid_write: &mut cpu::Callback)
    {
        self.last_sid_byte = value;
        match addr
        {
            // TODO
            0xD420...0xD7FF => self.write_register(0xD400 + (addr % 0x0020), value, on_sid_write),
            _               => as_ref!(self.mem_ref).get_ram_bank(memory::MemType::IO).write(addr, value)
        }

        *on_sid_write = cpu::Callback::None;
    }
    
    pub fn update(&mut self)
    {
        // TODO
    }
}
