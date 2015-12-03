// CIA
#![allow(dead_code)]
use c64::cpu;
use c64::vic;
use std::rc::Rc;
use std::cell::RefCell;

pub type CIAShared = Rc<RefCell<CIA>>;

pub enum CIACallbackAction
{
    CIA_NONE,
}


pub struct CIA
{
    cpu_ref: Option<cpu::CPUShared>,
    vic_ref: Option<vic::VICShared>,
}

impl CIA
{
    pub fn new_shared() -> CIAShared
    {
        Rc::new(RefCell::new(CIA
        {
            cpu_ref: None,
            vic_ref: None,
        }))
    }

    pub fn set_references(&mut self, cpuref: cpu::CPUShared, vicref: vic::VICShared)
    {
        self.cpu_ref = Some(cpuref);
        self.vic_ref = Some(vicref);
    }
    
    pub fn reset(&mut self)
    {
        // TODO
    }

    pub fn read_register(&self, addr: u16) -> u8
    {
        // TODO
        0
    }

    pub fn write_register(&mut self, addr: u16, value: u8, on_vic_write: &mut CIACallbackAction) -> bool
    {
        // TODO
        true
    }
    
    pub fn update(&mut self)
    {
        // TODO
    }
}
