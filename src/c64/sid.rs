// SID
//extern crate sdl2;
use c64::memory;

pub struct SID
{
    mem_ref: Option<memory::MemShared>,
}

impl SID
{
    pub fn new() -> SID
    {
        SID {
            mem_ref: None,
        }
    }

    pub fn set_references(&mut self, memref: memory::MemShared)
    {
        self.mem_ref = Some(memref);
    }

    pub fn update(&mut self)
    {
        // TODO
    }
}
