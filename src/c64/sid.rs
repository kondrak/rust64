// SID
use c64::memory;
use c64::cpu;
use std::cell::RefCell;
use std::rc::Rc;

pub type SIDShared = Rc<RefCell<SID>>;

pub struct SID
{
    mem_ref: Option<memory::MemShared>,
}

impl SID
{
    pub fn new_shared() -> SIDShared
    {
        Rc::new(RefCell::new(SID
        {
            mem_ref: None,
        }))
    }

    pub fn set_references(&mut self, memref: memory::MemShared)
    {
        self.mem_ref = Some(memref);
    }

    pub fn read_register(&self, addr: u16) -> u8
    {
        // TODO: for now return value stored in RAM
        as_ref!(self.mem_ref).read_byte(addr)
        
    }
    
    pub fn write_register(&mut self, addr: u16, value: u8, on_sid_write: &mut cpu::Callback)
    {
        // TODO
        as_mut!(self.mem_ref).write_byte(addr, value);
        *on_sid_write = cpu::Callback::None;
    }
    
    pub fn update(&mut self)
    {
        // TODO
    }
}
