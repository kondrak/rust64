// The CPU
#![allow(dead_code)]
#![allow(non_snake_case)]
extern crate sdl2;
use c64::opcodes;
use c64::memory;
use c64::vic;
use c64::cia;
use std::cell::RefCell;
use std::rc::Rc;

use utils;

pub type CPUShared = Rc<RefCell<CPU>>;


// status flags for P register
pub enum StatusFlag
{
    Negative         = 1 << 0,
    Overflow         = 1 << 1,
    Unused           = 1 << 2,
    Break            = 1 << 3,
    DecimalMode      = 1 << 4,
    InterruptDisable = 1 << 5,
    Zero             = 1 << 6,
    Carry            = 1 << 7 
}

pub static NMI_VECTOR:   u16 = 0xFFFA;
pub static RESET_VECTOR: u16 = 0xFFFC;
pub static IRQ_VECTOR:   u16 = 0xFFFE;

pub struct CPU
{
    pub PC: u16, // program counter
    pub SP: u8,  // stack pointer
    pub P: u8,   // processor status
    pub A: u8,   // accumulator
    pub X: u8,   // index register
    pub Y: u8,   // index register
    pub mem_ref: Option<memory::MemShared>, // reference to shared system memory
    pub vic_ref: Option<vic::VICShared>,
    pub cia1_ref: Option<cia::CIAShared>,
    pub cia2_ref: Option<cia::CIAShared>,
    pub ba_low: bool,  // is BA low?
    pub prev_PC: u16, // previous program counter - for debugging
    pub op_debugger: utils::OpDebugger
}

impl CPU
{
    pub fn new_shared() -> CPUShared
    {
        Rc::new(RefCell::new(CPU
        {
            PC: 0,
            SP: 0xFF,
            P: 0,
            A: 0,
            X: 0,
            Y: 0,
            mem_ref: None,
            vic_ref: None,
            cia1_ref: None,
            cia2_ref: None,
            ba_low: false,
            prev_PC: 0,
            op_debugger: utils::OpDebugger::new()
        }))
    }

    pub fn set_references(&mut self, memref: memory::MemShared, vicref: vic::VICShared, cia1ref: cia::CIAShared, cia2ref: cia::CIAShared)
    {
        self.mem_ref = Some(memref);
        self.vic_ref = Some(vicref);
        self.cia1_ref = Some(cia1ref);
        self.cia2_ref = Some(cia2ref);
    }    
    
    pub fn set_status_flag(&mut self, flag: StatusFlag, value: bool)
    {
        if value { self.P |=   flag as u8;  }
        else     { self.P &= !(flag as u8); }
    }

    pub fn get_status_flag(&mut self, flag: StatusFlag) -> bool
    {
        self.P & flag as u8 != 0x00
    }

    // these flags will be set in tandem quite often
    pub fn set_zn_flags(&mut self, value: u8)
    {
        self.set_status_flag(StatusFlag::Zero, value == 0x00);
        self.set_status_flag(StatusFlag::Negative, (value as i8) < 0);
    }
    
    pub fn reset(&mut self)
    {
        // reset program counter
        let pc = self.read_word_le(RESET_VECTOR);
        self.PC = pc;
        self.SP = 0xFF;
    }

    pub fn update(&mut self)
    {
        let op = self.next_byte();
        self.process_op(op);
        self.process_nmi();
        self.process_irq();
    }

    pub fn next_byte(&mut self) -> u8
    {
        let op = self.read_byte(self.PC);
        self.PC += 1;
        op
    }

    pub fn next_word(&mut self) -> u16
    {
        let word = self.read_word_le(self.PC);
        self.PC += 2;
        word
    }
    

    // stack memory: $0100 - $01FF (256 byes)
    // TODO: some extra message if stack over/underflow occurs? (right now handled by Rust)
    pub fn push_byte(&mut self, value: u8)
    {
        self.SP -= 0x01;
        let newSP = (self.SP + 0x01) as u16;
        self.write_byte(0x0100 + newSP, value);
    }

    pub fn pop_byte(&mut self) -> u8
    {
        let value = self.read_byte(0x0100 + (self.SP + 0x01) as u16);
        self.SP += 0x01;
        value
    }

    pub fn push_word(&mut self, value: u16)
    {
        self.SP -= 0x02;
        self.write_word_le(0x0100 + (self.SP + 0x01) as u16, value);
    }

    pub fn pop_word(&mut self) -> u16
    {
        let value = self.read_word_le(0x0100 + (self.SP + 0x01) as u16);
        self.SP += 0x02;
        value
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) -> bool
    {
        let mut on_vic_write: vic::VICCallbackAction = vic::VICCallbackAction::None;
        let mut on_cia_write: cia::CIACallbackAction = cia::CIACallbackAction::None;
        let mut mem_write_ok: bool;
        match addr
        {
            // VIC-II address space
            0xD000...0xD400 => mem_write_ok = as_mut!(self.vic_ref).write_register(addr, value, &mut on_vic_write),
            // CIA1 address space
            0xDC00...0xDCFF => mem_write_ok = as_mut!(self.cia1_ref).write_register(addr, value, &mut on_cia_write),
            // CIA2 address space
            0xDD00...0xDD0F => mem_write_ok = as_mut!(self.cia2_ref).write_register(addr, value, &mut on_cia_write),            
            _ => mem_write_ok = as_mut!(self.mem_ref).write_byte(addr, value),
        }

        // on VIC register write perform necessary action on the CPU
        match on_vic_write
        {
            vic::VICCallbackAction::TriggerVICIrq => self.trigger_vic_irq(),
            vic::VICCallbackAction::ClearVICIrq   => self.clear_vic_irq(),
            _ => (),
        }

        match on_cia_write
        {
            cia::CIACallbackAction::TriggerCIAIRQ => self.trigger_cia_irq(),
            cia::CIACallbackAction::ClearCIAIRQ   => self.clear_cia_irq(),
            cia::CIACallbackAction::TriggerNMI    => self.trigger_nmi(),
            cia::CIACallbackAction::ClearNMI      => self.clear_nmi(),            
            _ => (),
        }        

        mem_write_ok
    }
    
    pub fn read_byte(&self, addr: u16) -> u8
    {
        let mut byte: u8;
        let mut on_cia_read: cia::CIACallbackAction = cia::CIACallbackAction::None;
        
        match addr
        {
            // VIC-II address space
            0xD000...0xD400 => byte = as_ref!(self.vic_ref).read_register(addr),
            // CIA1 address space
            0xDC00...0xDCFF => byte = as_ref!(self.cia1_ref).read_register(addr, &mut on_cia_read),
            // CIA2 address space
            0xDD00...0xDD0F => byte = as_ref!(self.cia2_ref).read_register(addr, &mut on_cia_read),
            _ => byte = as_ref!(self.mem_ref).read_byte(addr)
        }

        /*match on_cia_read
        {
            cia::CIACallbackAction::TriggerCIAIRQ => self.trigger_cia_irq(),
            cia::CIACallbackAction::ClearCIAIRQ   => self.clear_cia_irq(),
            cia::CIACallbackAction::TriggerNMI    => self.trigger_nmi(),
            cia::CIACallbackAction::ClearNMI      => self.clear_nmi(),            
            _ => (),
        }*/

        byte
    }

    pub fn read_word_le(&self, addr: u16) -> u16
    {
        as_ref!(self.mem_ref).read_word_le(addr)
    }

    pub fn read_word_be(&self, addr: u16) -> u16
    {
        as_ref!(self.mem_ref).read_word_be(addr)
    }

    pub fn write_word_le(&self, addr: u16, value: u16) -> bool
    {
        as_ref!(self.mem_ref).write_word_le(addr, value)
    }
    
    fn process_nmi(&mut self)
    {
        // TODO: non-maskable irq
    }
    
    fn process_irq(&mut self)
    {
        if !self.get_status_flag(StatusFlag::InterruptDisable)
        {
            // TODO irq
        }
    }

    pub fn trigger_vic_irq(&mut self)
    {
        // TODO:
    }

    pub fn clear_vic_irq(&mut self)
    {
        // TODO
    }

    pub fn trigger_nmi(&mut self)
    {
        // TODO
    }

    pub fn clear_nmi(&mut self)
    {
        // TODO
    }

    pub fn trigger_cia_irq(&mut self)
    {
        // TODO
    }

    pub fn clear_cia_irq(&mut self)
    {
        // TODO
    }
    
    fn process_op(&mut self, opcode: u8) -> u8
    {
        //utils::debug_instruction(opcode, self);
        self.prev_PC = self.PC;
        match opcodes::get_instruction(opcode, self)
        {
            Some((instruction, num_cycles, addr_mode)) => {
                utils::debug_instruction(opcode, Some((&instruction, num_cycles, &addr_mode)), self);
                instruction.run(&addr_mode, self);
                num_cycles
            },
            None => panic!("No instruction - this should never happen! (0x{:02X} at ${:04X})", opcode, self.PC)
        }
    }
}
