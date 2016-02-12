// The CPU
#![allow(non_snake_case)]
//extern crate sdl2;
use c64::opcodes::*;
use c64::memory;
use c64::vic;
use c64::cia;
use std::cell::RefCell;
use std::rc::Rc;
use std::num::Wrapping;

use utils;

pub type CPUShared = Rc<RefCell<CPU>>;


// status flags for P register
pub enum StatusFlag
{
    Carry            = 1 << 0,
    Zero             = 1 << 1,
    InterruptDisable = 1 << 2,
    DecimalMode      = 1 << 3,
    Break            = 1 << 4,
    Unused           = 1 << 5,
    Overflow         = 1 << 6,
    Negative         = 1 << 7,
}

// action to perform on specific CIA and VIC events
pub enum CallbackAction
{
    None,
    TriggerVICIrq,
    ClearVICIrq,
    TriggerCIAIrq,
    ClearCIAIrq,
    TriggerNMI,
    ClearNMI
}

pub static NMI_VECTOR:   u16 = 0xFFFA;
pub static RESET_VECTOR: u16 = 0xFFFC;
pub static IRQ_VECTOR:   u16 = 0xFFFE;

enum CPUState
{
    FetchOp,
    FetchOperand,
    PerformRMW,
    ProcessIRQ,
    ProcessNMI,
    ExecuteOp
}

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
    pub curr_instr: Instruction,
    pub ba_low: bool,  // is BA low?
    pub cia_irq: bool,
    pub vic_irq: bool,
    pub irq_cycles_left: u8,
    pub nmi_cycles_left: u8,
    first_nmi_cycle: u32,
    first_irq_cycle: u32,
    state: CPUState,
    nmi: bool,
    pub debug_instr: bool,
    pub prev_PC: u16, // previous program counter - for debugging
    dfff_byte: u8,
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
            cia_irq: false,
            vic_irq: false,
            irq_cycles_left: 0,
            nmi_cycles_left: 0,
            first_nmi_cycle: 0,
            first_irq_cycle: 0,
            state: CPUState::FetchOp,
            curr_instr: Instruction::new(Op::BRK, 1, false, AddrMode::Implied),
            nmi: false,
            debug_instr: false,
            prev_PC: 0,
            dfff_byte: 0x55,
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
        self.ba_low = false;
        self.cia_irq = false;
        self.vic_irq = false;
        self.nmi = false;
    }

    pub fn update(&mut self, c64_cycle_cnt: u32)
    {
        // check for irq and nmi
        match self.state
        {
            CPUState::FetchOp => {
                if self.nmi && self.nmi_cycles_left == 0 && (c64_cycle_cnt - (self.first_nmi_cycle as u32) >= 2)
                {
                    self.nmi_cycles_left = 7;
                    self.state = CPUState::ProcessNMI;
                }
                else if (self.cia_irq || self.vic_irq) && self.irq_cycles_left == 0 && !self.get_status_flag(StatusFlag::InterruptDisable) && (c64_cycle_cnt - (self.first_irq_cycle as u32) >= 2)
                {
                    self.irq_cycles_left = 7;
                    self.state = CPUState::ProcessIRQ;
                }
            },
            _ => {}
        }
        
        match self.state
        {
            CPUState::FetchOp => {
                // fetch the op
                if self.ba_low { return; }
                let next_op = self.next_byte();
                match get_instruction(next_op) {
                    Some((op_name, total_cycles, is_rmw, addr_mode)) => {
                        self.curr_instr = Instruction::new(op_name, total_cycles, is_rmw, addr_mode);
                        if self.debug_instr { utils::debug_instruction(next_op, self); }
                    }
                    None => panic!("Can't fetch instruction")
                }
                
                // implied addressed mode instructions don't fetch operands
                match self.curr_instr.addr_mode {
                    AddrMode::Implied => self.state = CPUState::ExecuteOp,
                    AddrMode::Accumulator => {
                        //self.curr_instr.operand_value = self.A;
                        self.state = CPUState::ExecuteOp;
                    },
                    AddrMode::Immediate => {
                       // self.curr_instr.operand_value = self.next_byte() as u16;
                        self.state = CPUState::ExecuteOp;
                    },
                    AddrMode::Relative => {
                        // TODO: inc PC only during op execution?
                        let base = (self.PC + 1) as i16;
                        let offset = self.next_byte() as i8;
                        self.curr_instr.operand_addr = (base + offset as i16) as u16;
                        self.state = CPUState::ExecuteOp;
                    },
                    _ => self.state = CPUState::FetchOperand,
                };
            },
            CPUState::FetchOperand => {
                if self.fetch_operand()
                {
                    if self.curr_instr.is_rmw
                    {
                        self.state = CPUState::PerformRMW;
                    }
                    else
                    {
                        self.state = CPUState::ExecuteOp;
                    }
                }

                // TODO: odd case? Some instructions can be executed immediately after operand fetch
                if self.curr_instr.cycles_to_run == 0 && self.curr_instr.cycles_to_fetch == 0
                {
                    //panic!("Not sure if this should happen - reinvestigate");
                    self.run_instruction();
                    self.state = CPUState::FetchOp;
                }
            }
            CPUState::ProcessIRQ => {
                if self.process_irq()
                {
                    self.cia_irq = false;
                    self.vic_irq = false;
                    self.state = CPUState::FetchOp;
                }
            },
            CPUState::ProcessNMI => {
                if self.process_nmi()
                {
                    self.nmi = false;
                    self.state = CPUState::FetchOp;
                }
            },
            CPUState::PerformRMW => {
                match self.curr_instr.cycles_to_rmw
                {
                    2 => {
                        if self.ba_low { return; }
                        let addr = self.curr_instr.operand_addr;
                        self.curr_instr.rmw_buffer = self.read_byte(addr);
                    },
                    1 => {
                        let addr = self.curr_instr.operand_addr;
                        let val = self.curr_instr.rmw_buffer;
                        self.write_byte(addr, val);
                        self.state = CPUState::ExecuteOp;
                    },
                     _ => panic!("Too many cycles in RMW stage! ({}) ", self.curr_instr.cycles_to_rmw)
                }

                self.curr_instr.cycles_to_rmw -= 1;
            },
            CPUState::ExecuteOp => {
                if self.run_instruction()
                {
                    self.state = CPUState::FetchOp;
                }
            }
        }
        /*if self.process_nmi() { self.irq_cycles = 7; }
        else if self.process_irq() { self.irq_cycles = 7; }
        
        if !self.ba_low {

            if self.irq_cycles > 0
            {
                self.irq_cycles -= 1;
                return
            }
            
            if self.op_cycles == 0
            {
                self.curr_op = self.next_byte();
                let co = self.curr_op;
                self.op_cycles = self.get_op_cycles(co);
            }

            if self.op_cycles > 0
            {
                self.op_cycles -= 1;
            }

            if self.op_cycles == 0
            {
                let co = self.curr_op;
                self.process_op(co);
            }
        }*/
    }

    pub fn next_byte(&mut self) -> u8
    {
        let pc = self.PC;
        let op = self.read_byte(pc);
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
        let addr = 0x0100 + (self.SP + 0x01) as u16;
        let value = self.read_byte(addr);
        self.SP += 0x01;
        value
    }

    pub fn push_word(&mut self, value: u16)
    {
        self.push_byte(((value >> 8) & 0xFF) as u8);
        self.push_byte((value & 0xFF) as u8);
    }

    pub fn pop_word(&mut self) -> u16
    {
        let lo = (self.pop_byte() as u16) & 0x00FF;
        let hi = (self.pop_byte() as u16) & 0x00FF;
        (hi << 8) | lo
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) -> bool
    {
        let mut write_callback = CallbackAction::None;
        let mut mem_write_ok = true;
        let io_enabled = as_ref!(self.mem_ref).io_on;

        match addr
        {
            // VIC-II address space
            0xD000...0xD3FF => {
                if io_enabled
                {
                    as_mut!(self.vic_ref).write_register(addr, value, &mut write_callback);
                }
                else
                {
                    mem_write_ok = as_mut!(self.mem_ref).write_byte(addr, value);
                }
            },
            // color RAM address space
            0xD800...0xDBFF => {
                if io_enabled
                {
                    mem_write_ok = as_mut!(self.mem_ref).write_byte(addr, value & 0x0F);
                }
                else
                {
                    mem_write_ok = as_mut!(self.mem_ref).write_byte(addr, value);
                }
            },
            // CIA1 address space
            0xDC00...0xDCFF => {
                if io_enabled
                {
                    as_mut!(self.cia1_ref).write_register(addr, value, &mut write_callback);
                }
                else
                {
                    mem_write_ok = as_mut!(self.mem_ref).write_byte(addr, value);
                }
            },
            // CIA2 address space
            0xDD00...0xDDFF => {
                if io_enabled
                {
                    as_mut!(self.cia2_ref).write_register(addr, value, &mut write_callback);
                }
                else
                {
                    mem_write_ok = as_mut!(self.mem_ref).write_byte(addr, value);
                }
            },
            _ => mem_write_ok = as_mut!(self.mem_ref).write_byte(addr, value),
        }

        // on VIC/CIA register write perform necessary action on the CPU
        match write_callback
        {
            CallbackAction::TriggerVICIrq => self.trigger_vic_irq(),
            CallbackAction::ClearVICIrq   => self.clear_vic_irq(),
            CallbackAction::TriggerCIAIrq => self.trigger_cia_irq(),
            CallbackAction::ClearCIAIrq   => self.clear_cia_irq(),
            CallbackAction::TriggerNMI    => self.trigger_nmi(),
            CallbackAction::ClearNMI      => self.clear_nmi(),
            _ => (),
        }

        mem_write_ok
    }

    pub fn read_idle(&mut self, addr: u16)
    {
        let _ = self.read_byte(addr);
    }
    
    pub fn read_byte(&mut self, addr: u16) -> u8
    {
        let byte: u8;
        let mut read_callback = CallbackAction::None;
        let io_enabled = as_ref!(self.mem_ref).io_on;
        match addr
        {
            // VIC-II address space
            0xD000...0xD3FF => {
                if io_enabled
                {
                    byte = as_mut!(self.vic_ref).read_register(addr);
                }
                else
                {
                    byte = as_mut!(self.mem_ref).read_byte(addr);
                }
            },
            // color RAM address space
            0xD800...0xDBFF => {
                if io_enabled
                {
                    byte = (as_ref!(self.mem_ref).read_byte(addr) & 0x0F) | (as_ref!(self.vic_ref).last_byte & 0xF0);
                }
                else
                {
                    byte = as_mut!(self.mem_ref).read_byte(addr);
                }
            },
            // CIA1 address space
            0xDC00...0xDCFF => {
                if io_enabled
                {
                    byte = as_mut!(self.cia1_ref).read_register(addr, &mut read_callback);
                }
                else
                {
                    byte = as_mut!(self.mem_ref).read_byte(addr);
                }
            },
            // CIA2 address space
            0xDD00...0xDDFF => {
                if io_enabled
                {
                    byte = as_mut!(self.cia2_ref).read_register(addr, &mut read_callback);
                }
                else
                {
                    byte = as_mut!(self.mem_ref).read_byte(addr);
                }
            },
            0xDF00...0xDF9F => {
                if io_enabled
                {
                    byte = as_ref!(self.vic_ref).last_byte;
                }
                else
                {
                    byte = as_mut!(self.mem_ref).read_byte(addr);
                }
            },
            0xDFFF => {
                if io_enabled
                {
                    self.dfff_byte = !self.dfff_byte;
                    byte = self.dfff_byte;
                }
                else
                {
                    byte = as_mut!(self.mem_ref).read_byte(addr);
                }
            }, 
            _ => byte = as_mut!(self.mem_ref).read_byte(addr)
        }

        match read_callback
        {
            CallbackAction::TriggerCIAIrq => self.trigger_cia_irq(),
            CallbackAction::ClearCIAIrq   => self.clear_cia_irq(),
            CallbackAction::TriggerNMI    => self.trigger_nmi(),
            CallbackAction::ClearNMI      => self.clear_nmi(),
            _ => (),
        }

        byte
    }

    pub fn read_word_le(&self, addr: u16) -> u16
    {
        as_ref!(self.mem_ref).read_word_le(addr)
    }

    pub fn write_word_le(&self, addr: u16, value: u16) -> bool
    {
        as_ref!(self.mem_ref).write_word_le(addr, value)
    }
    
    fn process_nmi(&mut self) -> bool
    {
        // only process irq if it's the "fetch op" stage
        /*if self.op_cycles != 0 { return false }
        // 7 cycles
        if self.nmi
        {
            let curr_pc = self.PC;
            let curr_p = self.P;
            self.push_word(curr_pc);
            self.push_byte(curr_p);
            self.set_status_flag(StatusFlag::InterruptDisable, true);
            self.PC = as_ref!(self.mem_ref).read_word_le(NMI_VECTOR);
            self.nmi = false;
            true
        }
        else
        {
            false
        } */

        match self.nmi_cycles_left
        {
            7 => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
            },
            6 => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
            },
            5 => {
                let pc_hi = (self.PC >> 8) as u8;
                self.push_byte(pc_hi);
            },
            4 => {
                let pc_lo = self.PC as u8;
                self.push_byte(pc_lo);
            },
            3 => {
                //self.set_status_flag(StatusFlag::Break, false); // TODO: clear brk flag?
                let curr_p = self.P;
                self.push_byte(curr_p);
                self.set_status_flag(StatusFlag::InterruptDisable, true);
            },
            2 => {
                if self.ba_low { return false; } // TODO: is reading whole word ok in cycle 1?
            },
            1 => {
                if self.ba_low { return false; }
                self.PC = as_ref!(self.mem_ref).read_word_le(NMI_VECTOR);
            }
            _ => panic!("Invalid NMI cycle")
        }

        self.nmi_cycles_left -= 1;
        self.nmi_cycles_left == 0
    }
    
    fn process_irq(&mut self) -> bool
    {
        // only process irq if it's the "fetch op" stage
        //if self.op_cycles != 0 { return false }
        // 7 cycles
        /*if (self.cia_irq || self.vic_irq) && !self.get_status_flag(StatusFlag::InterruptDisable)
        {
            self.set_status_flag(StatusFlag::Break, false);
            let curr_pc = self.PC;
            let curr_p = self.P;
            //println!("PC {} P {}", curr_pc, curr_p);
            self.push_word(curr_pc);
            self.push_byte(curr_p);
            self.set_status_flag(StatusFlag::InterruptDisable, true);
            self.PC = as_ref!(self.mem_ref).read_word_le(IRQ_VECTOR);
            self.cia_irq = false;
            self.vic_irq = false;
            true
        } */

        match self.irq_cycles_left
        {
            7 => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
            },
            6 => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
            },
            5 => {
                let pc_hi = (self.PC >> 8) as u8;
                self.push_byte(pc_hi);
            },
            4 => {
                let pc_lo = self.PC as u8;
                self.push_byte(pc_lo);
            },
            3 => {
                self.set_status_flag(StatusFlag::Break, false);
                let curr_p = self.P;
                self.push_byte(curr_p);
                self.set_status_flag(StatusFlag::InterruptDisable, true);
            },
            2 => {
                if self.ba_low { return false; } // TODO: is reading whole word ok in cycle 1?
            },
            1 => {
                if self.ba_low { return false; }
                self.PC = as_ref!(self.mem_ref).read_word_le(IRQ_VECTOR);
            }
            _ => panic!("Invalid IRQ cycle")
        }

        self.irq_cycles_left -= 1;
        self.irq_cycles_left == 0
    }

    pub fn trigger_vic_irq(&mut self)
    {
        //println!("VIC irq triggered");
        self.vic_irq = true;
    }

    pub fn clear_vic_irq(&mut self)
    {
        self.vic_irq = false;
    }

    pub fn trigger_nmi(&mut self)
    {
        //println!("NMI irq");
        self.nmi = true;
    }

    pub fn clear_nmi(&mut self)
    {
        self.nmi = false;
    }

    pub fn trigger_cia_irq(&mut self)
    {
        //println!("CIA irq triggered");
        self.cia_irq = true;
    }

    pub fn clear_cia_irq(&mut self)
    {
        self.cia_irq = false;
    }
    
    fn process_op(&mut self, opcode: u8) -> u8
    {
        //utils::debug_instruction(opcode, self);
      /*  self.prev_PC = self.PC;
        match opcodes::get_instruction(opcode, self)
        {
            Some((instruction, num_cycles, addr_mode)) => {
                //utils::debug_instruction(opcode, Some((&instruction, num_cycles, &addr_mode)), self);
                instruction.run(&addr_mode, self);
                num_cycles
            },
            None => panic!("No instruction - this should never happen! (0x{:02X} at ${:04X})", opcode, self.PC)
    } */
        0
    }

    
    fn fetch_operand(&mut self) -> bool
    {
        if self.ba_low { return false; }
        
        match self.curr_instr.addr_mode
        {
            AddrMode::Absolute => {
                match self.curr_instr.cycles_to_fetch {
                    2 => {
                        self.curr_instr.operand_addr = self.next_byte() as u16;
                    },
                    1 => {
                        self.curr_instr.operand_addr = self.curr_instr.operand_addr | ((self.next_byte() as u16) << 8);
                    },
                    _ => panic!("Too many cycles for operand address fetch! ({}) ", self.curr_instr.cycles_to_fetch)
                }
            },
            AddrMode::AbsoluteIndexedX(extra_cycle) => {
                match self.curr_instr.cycles_to_fetch {
                    3 => {
                        self.curr_instr.operand_addr = self.next_byte() as u16;
                    },
                    2 => {
                        let addr_lo = self.curr_instr.operand_addr;
                        self.curr_instr.index_addr = self.next_byte() as u16;
                        self.curr_instr.operand_addr = ((addr_lo + self.X as u16) & 0xFF) | (self.curr_instr.index_addr << 8);
                        // page crossed?
                        self.curr_instr.zp_crossed = addr_lo + (self.X as u16) >= 0x100;

                        // if instruction has extra cycle on page crossing and it hasn't happened, we don't get
                        // the extra cycle (finish fetching now)
                        if !self.curr_instr.zp_crossed && extra_cycle
                        {
                            self.curr_instr.cycles_to_fetch = 1;
                        }
                    },
                    1 => { // if page crossed - add 0x100 to operand address
                        let addr = self.curr_instr.operand_addr;
                        self.read_idle(addr);                     
                        if self.curr_instr.zp_crossed { self.curr_instr.operand_addr += 0x100; }
                    },
                    _ => panic!("Too many cycles for operand address fetch! ({}) ", self.curr_instr.cycles_to_fetch)
                }
            },
            AddrMode::AbsoluteIndexedY(extra_cycle) => {
                match self.curr_instr.cycles_to_fetch {
                    3 => {
                        self.curr_instr.operand_addr = self.next_byte() as u16;
                    },
                    2 => {
                        self.curr_instr.index_addr = self.next_byte() as u16;
                        let addr_lo = self.curr_instr.operand_addr;
                        self.curr_instr.operand_addr = ((addr_lo + self.Y as u16) & 0xFF) | (self.curr_instr.index_addr << 8);
                        // page crossed?
                        self.curr_instr.zp_crossed = addr_lo + (self.Y as u16) >= 0x100;
                        
                        // if instruction has extra cycle on page crossing and it hasn't happened, we don't get
                        // the extra cycle (finish fetching now)
                        if !self.curr_instr.zp_crossed && extra_cycle
                        {
                            self.curr_instr.cycles_to_fetch = 1;
                        }
                    },
                    1 => { // if page crossed - add 0x100 to operand address
                        let addr = self.curr_instr.operand_addr;
                        self.read_idle(addr);
                        if self.curr_instr.zp_crossed { self.curr_instr.operand_addr += 0x100; }
                    },
                    _ => panic!("Too many cycles for operand address fetch! ({}) ", self.curr_instr.cycles_to_fetch)
                }
            },
            AddrMode::Zeropage => {
                self.curr_instr.operand_addr = self.next_byte() as u16;
            },
            AddrMode::ZeropageIndexedX => {
                match self.curr_instr.cycles_to_fetch {
                    2 => {
                        self.curr_instr.operand_addr = self.next_byte() as u16;
                    },
                    1 => {
                        let x = self.X as u16;
                        let base_addr = self.curr_instr.operand_addr;
                        self.read_idle(base_addr);
                        self.curr_instr.operand_addr = ((Wrapping(base_addr) + Wrapping(x)).0 as u16) & 0xFF;
                    }
                    _ => panic!("Too many cycles for operand address fetch! ({}) ", self.curr_instr.cycles_to_fetch)
                }
            },
            AddrMode::ZeropageIndexedY => {
                match self.curr_instr.cycles_to_fetch {
                    2 => {
                        self.curr_instr.operand_addr = self.next_byte() as u16;
                    },
                    1 => {
                        let y = self.Y as u16;
                        let base_addr = self.curr_instr.operand_addr;
                        self.read_idle(base_addr);
                        self.curr_instr.operand_addr = ((Wrapping(base_addr) + Wrapping(y)).0 as u16) & 0xFF;
                    }
                    _ => panic!("Too many cycles for operand address fetch! ({}) ", self.curr_instr.cycles_to_fetch)
                }
            },
            AddrMode::IndexedIndirectX => {
                match self.curr_instr.cycles_to_fetch {
                    4 => {
                        self.curr_instr.index_addr = self.next_byte() as u16;
                    },
                    3 => {
                        let addr = self.curr_instr.index_addr;
                        self.read_idle(addr);
                        self.curr_instr.index_addr = (self.curr_instr.index_addr + self.X as u16) & 0xFF;
                    },
                    2 => {
                        let idx_addr = self.curr_instr.index_addr;
                        self.curr_instr.operand_addr =  self.read_byte(idx_addr) as u16;
                    },
                    1 => {
                        let idx = self.curr_instr.index_addr;
                        let hi = self.read_byte((idx + 1) & 0xFF) as u16;
                        self.curr_instr.operand_addr = self.curr_instr.operand_addr | (hi << 8);
                    },
                    _ => panic!("Too many cycles for operand address fetch! ({}) ", self.curr_instr.cycles_to_fetch)
                }
            },
            AddrMode::IndirectIndexedY(extra_cycle) => {
                match self.curr_instr.cycles_to_fetch {
                    4 => {
                        self.curr_instr.index_addr = self.next_byte() as u16;
                    },
                    3 => {
                        let base_addr = self.curr_instr.index_addr;
                        self.curr_instr.operand_addr = self.read_byte(base_addr) as u16;
                    },
                    2 => {
                        let idx = self.curr_instr.index_addr;
                        let opaddr = self.curr_instr.operand_addr;
                        self.curr_instr.index_addr =  self.read_byte((idx + 1) & 0xFF ) as u16;
                        self.curr_instr.operand_addr = ((opaddr + self.Y as u16) & 0x0FF) | (self.curr_instr.index_addr << 8);
                        // page crossed?
                        self.curr_instr.zp_crossed = opaddr + (self.Y as u16) >= 0x100;

                        // if instruction has extra cycle on page crossing and it hasn't happened, we don't get
                        // the extra cycle (finish fetching now)
                        if !self.curr_instr.zp_crossed && extra_cycle
                        {
                            self.curr_instr.cycles_to_fetch = 1;
                        }
                    },
                    1 => { // if page crossed - add 0x100 to operand address
                        let addr = self.curr_instr.operand_addr;
                        self.read_idle(addr);
                        if self.curr_instr.zp_crossed { self.curr_instr.operand_addr += 0x100; }
                    },
                    _ => panic!("Too many cycles for operand address fetch! ({}) ", self.curr_instr.cycles_to_fetch)
                }
            },
            AddrMode::Indirect => {
                match self.curr_instr.cycles_to_fetch {
                    2 => {
                        self.curr_instr.operand_addr = self.next_byte() as u16;
                    },
                    1 => {
                        let addr = self.curr_instr.operand_addr | ((self.next_byte() as u16) << 8);
                        self.curr_instr.operand_addr = self.read_word_le(addr);
                    },
                    _ => panic!("Too many cycles for operand address fetch! ({}) ", self.curr_instr.cycles_to_fetch)
                }
            },
            _ => {}
        }

        self.curr_instr.cycles_to_fetch -= 1;
        // fetch complete
        self.curr_instr.cycles_to_fetch == 0
    }

    fn get_operand(&mut self) -> u8
    {
        let addr = self.curr_instr.operand_addr;

        if self.curr_instr.is_rmw
        {
            return self.curr_instr.rmw_buffer;
        }
        
        let val = match self.curr_instr.addr_mode {
            AddrMode::Accumulator => self.A,
            AddrMode::Immediate  => self.next_byte(),
            _ => self.read_byte(addr)
        };

        val
    }

    fn set_operand(&mut self, val: u8)
    {
        let addr = self.curr_instr.operand_addr;
        
        match self.curr_instr.addr_mode {
            AddrMode::Accumulator => { self.A = val; },
            _ => { self.write_byte(addr, val); },
        }
    }

    fn run_instruction(&mut self) -> bool
    {
        match self.curr_instr.op
        {
            Op::LDA => {
                if self.ba_low { return false; }
                let na = self.get_operand();
                self.A = na;
                self.set_zn_flags(na);
            },
            Op::LDX => {
                if self.ba_low { return false; }
                let nx = self.get_operand();
                self.X = nx;
                self.set_zn_flags(nx);
            },
            Op::LDY => {
                if self.ba_low { return false; }
                let ny = self.get_operand();
                self.Y = ny;
                self.set_zn_flags(ny);
            },
            Op::STA => {
                let addr = self.curr_instr.operand_addr;
                let val = self.A;
                self.write_byte(addr, val);
            },
            Op::STX => {
                let addr = self.curr_instr.operand_addr;
                let val = self.X;
                self.write_byte(addr, val);
            },
            Op::STY => {
                let addr = self.curr_instr.operand_addr;
                let val = self.Y;
                self.write_byte(addr, val);
            },
            Op::TAX => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.X = self.A;
                let x = self.X;
                self.set_zn_flags(x);
            },
            Op::TAY => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.Y = self.A;
                let y = self.Y;
                self.set_zn_flags(y);
            },
            Op::TXA => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.A = self.X;
                let a = self.A;
                self.set_zn_flags(a);
            },
            Op::TYA => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.A = self.Y;
                let a = self.A;
                self.set_zn_flags(a);
            },
            Op::TSX => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.X = self.SP;
                let x = self.X;
                self.set_zn_flags(x);
            },
            Op::TXS => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.SP = self.X;
            },
            Op::PHA => {
                match self.curr_instr.cycles_to_run
                {
                    2 => {
                        if self.ba_low { return false; }
                        let pc = self.PC;
                        self.read_idle(pc);
                    },
                    1 => {
                        let a = self.A;
                        self.push_byte(a);
                    },
                    _ => panic!("Wrong number of cycles: {} {}", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::PHP => {
                match self.curr_instr.cycles_to_run
                {
                    2 => {
                        if self.ba_low { return false; }
                        let pc = self.PC;
                        self.read_idle(pc);
                    },
                    1 => {
                        let p = self.P;
                        // TODO: break flag?
                        self.push_byte(p);
                    },
                    _ => panic!("Wrong number of cycles: {} {}", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::PLA => {
                if self.ba_low { return false; }
                match self.curr_instr.cycles_to_run
                {
                    3 => {
                        let pc = self.PC;
                        self.read_idle(pc);
                    },
                    2 => {
                        let sp = self.SP as u16;
                        self.read_idle(sp+1);
                    },
                    1 => {
                        let a = self.pop_byte();
                        self.A = a;
                        self.set_zn_flags(a);
                    },
                    _ => panic!("Wrong number of cycles: {} {}", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::PLP => {
                if self.ba_low { return false; }
                match self.curr_instr.cycles_to_run
                {
                    3 => {
                        let pc = self.PC;
                        self.read_idle(pc);
                    },
                    2 => {
                        let sp = self.SP as u16;
                        self.read_idle(sp+1);
                    },
                    1 => {
                        let p = self.pop_byte();
                        self.P = p;
                    },
                    _ => panic!("Wrong number of cycles: {} {}", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::AND => {
                if self.ba_low { return false; }
                let v = self.get_operand();
                let na = self.A & v;
                self.A = na;
                self.set_zn_flags(na);
            },
            Op::EOR => {
                if self.ba_low { return false; }
                let v = self.get_operand();
                let na = self.A ^ v;
                self.A = na;
                self.set_zn_flags(na);
            },
            Op::ORA => {
                if self.ba_low { return false; }
                let v = self.get_operand();
                let na = self.A | v;
                self.A = na;
                self.set_zn_flags(na);
            },
            Op::BIT => {
                if self.ba_low { return false; }
                let v = self.get_operand();
                let a = self.A;
                self.set_status_flag(StatusFlag::Negative, (v & 0x80) != 0); // TODO: is this ok?
                self.set_status_flag(StatusFlag::Overflow, (v & 0x40) != 0);
                self.set_status_flag(StatusFlag::Zero,     (v & a)    == 0);
            },
            Op::ADC => { // TODO: test decimal mode, check if flag values are correct
                if self.ba_low { return false; }
                let v = self.get_operand();
                let c = self.get_status_flag(StatusFlag::Carry);
                
                if self.get_status_flag(StatusFlag::DecimalMode)
                {
                    let mut lo = (Wrapping((self.A as u16) & 0xF) + Wrapping((v as u16) & 0xF)).0;
                    if  c { lo = (Wrapping(lo) + Wrapping(1)).0; }
                    if lo > 9 { lo = (Wrapping(lo) + Wrapping(6)).0; }

                    let mut hi = (Wrapping((self.A as u16) >> 4) + Wrapping((v as u16) >> 4)).0;
                    if lo > 0xF { hi = (Wrapping(hi) + Wrapping(1)).0; }

                    let is_overflow = ((((hi << 4) ^ (self.A as u16)) & 0x80) != 0) && (((self.A ^ v) & 0x80) == 0);
                    let mut is_zero = (Wrapping(self.A as u16) + Wrapping(v as u16)).0;
                    if c  { is_zero = (Wrapping(is_zero) + Wrapping(1)).0; }
                    
                    self.set_status_flag(StatusFlag::Negative, (hi << 4) != 0); // TODO: is this ok?              
                    self.set_status_flag(StatusFlag::Overflow, is_overflow);
                    self.set_status_flag(StatusFlag::Zero,     is_zero == 0);

                    if hi > 9 { hi = (Wrapping(hi) + Wrapping(6)).0; }
                    self.set_status_flag(StatusFlag::Carry, hi > 0xF);
                    self.A = ((hi << 4) | (lo & 0xF)) as u8;
                }
                else
                {
                    // TODO: should operation wrap automatically here?
                    let mut res: u16 = (Wrapping(self.A as u16) + Wrapping(v as u16)).0;
                    if c
                    {
                        res = (Wrapping(res) + Wrapping(0x0001)).0;
                    }
                    self.set_status_flag(StatusFlag::Carry, (res & 0x0100) != 0);
                    let res = res as u8;
                    let is_overflow = (self.A ^ v) & 0x80 == 0 && (self.A ^ res) & 0x80 == 0x80;
                    self.set_status_flag(StatusFlag::Overflow, is_overflow);
                    self.A = res;
                    self.set_zn_flags(res);
                }
            },
            Op::SBC => { // TODO: test decimal mode, check if flag values are correct
                if self.ba_low { return false; }
                
                let v = self.get_operand();
                let mut res: u16 = (Wrapping(self.A as u16) - Wrapping(v as u16)).0;
                if !self.get_status_flag(StatusFlag::Carry)
                {
                    res = (Wrapping(res) - Wrapping(0x0001)).0;
                }
                
                if self.get_status_flag(StatusFlag::DecimalMode)
                {
                    let mut lo = (Wrapping((self.A as u16) & 0xF) - Wrapping((v as u16) & 0xF)).0;
                    let mut hi = (Wrapping((self.A as u16) >> 4) - Wrapping((v as u16) >> 4)).0;

                    if !self.get_status_flag(StatusFlag::Carry)
                    {
                        lo = (Wrapping(lo) - Wrapping(1)).0;
                    }
                    
                    if (lo & 0x10) != 0
                    {
                        lo = (Wrapping(lo) - Wrapping(6)).0;
                        hi = (Wrapping(hi) - Wrapping(1)).0;
                    }

                    if (hi & 0x10) != 0 { hi = (Wrapping(hi) - Wrapping(6)).0; }

                    self.set_status_flag(StatusFlag::Carry, (res & 0x0100) == 0);
                    let res = res as u8;
                    let is_overflow = (self.A ^ res) & 0x80 != 0 && (self.A ^ v) & 0x80 == 0x80;
                    self.set_status_flag(StatusFlag::Overflow, is_overflow);
                    self.set_zn_flags(res);

                    self.A = ((hi << 4) | (lo & 0xF)) as u8;
                }
                else
                {
                    // TODO: should operation wrap automatically here?
                    self.set_status_flag(StatusFlag::Carry, (res & 0x0100) == 0);
                    let res = res as u8;
                    let is_overflow = (self.A ^ res) & 0x80 != 0 && (self.A ^ v) & 0x80 == 0x80;
                    self.set_status_flag(StatusFlag::Overflow, is_overflow);
                    self.A = res;
                    self.set_zn_flags(res);
                }
            },
            Op::CMP => {
                if self.ba_low { return false; }
                let v = self.get_operand();
                let res = self.A as i16 - v as i16;
                self.set_status_flag(StatusFlag::Carry, res >= 0);
                self.set_zn_flags(res as u8);
            },
            Op::CPX => {
                if self.ba_low { return false; }
                let v = self.get_operand();
                let res = self.X as i16 - v as i16;
                self.set_status_flag(StatusFlag::Carry, res >= 0);
                self.set_zn_flags(res as u8);
            },
            Op::CPY => {
                if self.ba_low { return false; }
                let v = self.get_operand();
                let res = self.Y as i16 - v as i16;
                self.set_status_flag(StatusFlag::Carry, res >= 0);
                self.set_zn_flags(res as u8);
            },
            Op::INC => {
                let v = (Wrapping(self.curr_instr.rmw_buffer) + Wrapping(0x01)).0;
                let addr = self.curr_instr.operand_addr;
                self.write_byte(addr, v);
                self.set_zn_flags(v);
            },
            Op::INX => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.X = (Wrapping(self.X) + Wrapping(0x01)).0;
                let x = self.X;
                self.set_zn_flags(x);
            },
            Op::INY => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.Y = (Wrapping(self.Y) + Wrapping(0x01)).0;
                let y = self.Y;
                self.set_zn_flags(y);
            },
            Op::DEC => {
                let v = (Wrapping(self.curr_instr.rmw_buffer) - Wrapping(0x01)).0;
                let addr = self.curr_instr.operand_addr;
                self.write_byte(addr, v);
                self.set_zn_flags(v);
            },
            Op::DEX => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.X = (Wrapping(self.X) - Wrapping(0x01)).0;
                let x = self.X;
                self.set_zn_flags(x);
            },
            Op::DEY => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.Y = (Wrapping(self.Y) - Wrapping(0x01)).0;
                let y = self.Y;
                self.set_zn_flags(y);
            },
            Op::ASL => {
                if self.ba_low {
                    match self.curr_instr.addr_mode {
                        AddrMode::Accumulator => {return false; },
                        _ => (),
                    }
                }
                let v = self.get_operand();
                self.set_status_flag(StatusFlag::Carry, (v & 0x80) != 0);
                let res = v << 1;
                self.set_operand(res);
                self.set_zn_flags(res);
            },
            Op::LSR => {
                if self.ba_low {
                    match self.curr_instr.addr_mode {
                        AddrMode::Accumulator => {return false; },
                        _ => (),
                    }
                }
                let v = self.get_operand();
                self.set_status_flag(StatusFlag::Carry, (v & 0x01) != 0);
                let res = v >> 1;
                self.set_operand(res);
                self.set_zn_flags(res);
            },
            Op::ROL => {
                if self.ba_low {
                    match self.curr_instr.addr_mode {
                        AddrMode::Accumulator => {return false; },
                        _ => (),
                    }
                }
                let c = self.get_status_flag(StatusFlag::Carry);
                let v = self.get_operand();
                self.set_status_flag(StatusFlag::Carry, (v & 0x80) != 0);
                let mut res = v << 1;
                if c
                {
                    res |= 0x01;
                }
                self.set_operand(res);
                self.set_zn_flags(res);
            },
            Op::ROR => {
                if self.ba_low {
                    match self.curr_instr.addr_mode {
                        AddrMode::Accumulator => {return false; },
                        _ => (),
                    }
                }
                let c = self.get_status_flag(StatusFlag::Carry);
                let v = self.get_operand();
                self.set_status_flag(StatusFlag::Carry, (v & 0x01) != 0);
                let mut res = v >> 1;
                if c
                {
                    res |= 0x80;
                }
                self.set_operand(res);
                self.set_zn_flags(res);
            },
            Op::JMP => { // TODO: is this ok?
                if self.ba_low { return false; }
                let pc = self.PC;
                match self.curr_instr.cycles_to_run
                {
                    2 => {
                        //self.curr_instr.cycles_to_run -= 1;
                    },
                    1 | 0 => {
                        self.PC = self.curr_instr.operand_addr;
                        self.curr_instr.cycles_to_run = 1;
                    },
                    _ => panic!("Wrong number of cycles: {} {} ", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::JSR => { // TODO: is this ok?
                match self.curr_instr.cycles_to_run
                {
                    3 => {
                        // TODO: break down PC push to 2 byte instructions?
                    },
                    2 => {
                        let pc = self.PC - 0x0001;
                        self.push_word(pc);
                    },
                    1  => {
                        if self.ba_low { return false; }
                        let pc = self.PC;
                        self.read_idle(pc);
                        self.PC = self.curr_instr.operand_addr;
                    },
                    _ => panic!("Wrong number of cycles: {} {} ", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::RTS => {
                if self.ba_low { return false; }

                match self.curr_instr.cycles_to_run
                {
                    5 => {
                        let pc = self.PC;
                        self.read_idle(pc);
                    },
                    4 => {
                        let sp = self.SP as u16;
                        self.read_idle(sp + 1);
                    },
                    3 => {
                        let pc_lo = self.pop_byte() as u16;
                        self.PC = pc_lo;
                    },
                    2 => {
                        let pc_hi = self.pop_byte() as u16;
                        self.PC |= pc_hi << 8;
                    },
                    1  => {
                        let pc = self.PC;
                        self.read_idle(pc+1);
                        self.PC += 1;
                    },
                    _ => panic!("Wrong number of cycles: {} {} ", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            // branching ops: (TODO: take into account forward/back branching?)
            // take 2 cycles (fetch + execute) if no branch is taken
            // 3 cycles if branch is taken, no page crossed
            // 4 cycles if branch is taken, page crossed
            Op::BCC => {
                match self.curr_instr.cycles_to_run
                {
                    3 => {
                        if self.ba_low { return false; }
                        if !self.get_status_flag(StatusFlag::Carry)
                        {
                            let addr = self.curr_instr.operand_addr;
                            let pc = self.PC;
                            self.curr_instr.zp_crossed = (addr >> 8) != (pc >> 8);
                        }
                        else
                        {
                            // no branching - finish instruction after only 2 cycles
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    2 => {
                        if !self.curr_instr.zp_crossed
                        {
                            self.first_irq_cycle += 1;
                            self.first_nmi_cycle += 1;
                        }
                        if self.ba_low { return false; }
                        
                        let pc = self.PC;
                        let addr = self.curr_instr.operand_addr;
                        self.read_idle(pc);
                        self.PC = addr;

                        if !self.curr_instr.zp_crossed
                        {
                            // no page crossing - finish instruction after only 3 cycle
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    1 => {
                        if self.ba_low { return false; }
                        let pc = self.PC;
                        self.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                    },
                    _ => panic!("Wrong number of cycles: {} {} ", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::BCS => {
                match self.curr_instr.cycles_to_run
                {
                    3 => {
                        if self.ba_low { return false; }
                        if self.get_status_flag(StatusFlag::Carry)
                        {
                            let addr = self.curr_instr.operand_addr;
                            let pc = self.PC;
                            self.curr_instr.zp_crossed = (addr >> 8) != (pc >> 8);
                        }
                        else
                        {
                            // no branching - finish instruction after only 2 cycles
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    2 => {
                        if !self.curr_instr.zp_crossed
                        {
                            self.first_irq_cycle += 1;
                            self.first_nmi_cycle += 1;
                        }
                        if self.ba_low { return false; }
                        
                        let pc = self.PC;
                        let addr = self.curr_instr.operand_addr;
                        self.read_idle(pc);
                        self.PC = addr;

                        if !self.curr_instr.zp_crossed
                        {
                            // no page crossing - finish instruction after only 3 cycle
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    1 => {
                        if self.ba_low { return false; }
                        let pc = self.PC;
                        self.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                    },
                    _ => panic!("Wrong number of cycles: {} {} ", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::BEQ => {
                match self.curr_instr.cycles_to_run
                {
                    3 => {
                        if self.ba_low { return false; }
                        if self.get_status_flag(StatusFlag::Zero)
                        {
                            let addr = self.curr_instr.operand_addr;
                            let pc = self.PC;
                            self.curr_instr.zp_crossed = (addr >> 8) != (pc >> 8);
                        }
                        else
                        {
                            // no branching - finish instruction after only 2 cycles
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    2 => {
                        if !self.curr_instr.zp_crossed
                        {
                            self.first_irq_cycle += 1;
                            self.first_nmi_cycle += 1;
                        }
                        if self.ba_low { return false; }
                        
                        let pc = self.PC;
                        let addr = self.curr_instr.operand_addr;
                        self.read_idle(pc);
                        self.PC = addr;

                        if !self.curr_instr.zp_crossed
                        {
                            // no page crossing - finish instruction after only 3 cycle
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    1 => {
                        if self.ba_low { return false; }
                        let pc = self.PC;
                        self.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                    },
                    _ => panic!("Wrong number of cycles: {} {} ", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::BNE => {
                match self.curr_instr.cycles_to_run
                {
                    3 => {
                        if self.ba_low { return false; }
                        if !self.get_status_flag(StatusFlag::Zero)
                        {
                            let addr = self.curr_instr.operand_addr;
                            let pc = self.PC;
                            self.curr_instr.zp_crossed = (addr >> 8) != (pc >> 8);
                        }
                        else
                        {
                            // no branching - finish instruction after only 2 cycles
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    2 => {
                        if !self.curr_instr.zp_crossed
                        {
                            self.first_irq_cycle += 1;
                            self.first_nmi_cycle += 1;
                        }
                        if self.ba_low { return false; }
                        
                        let pc = self.PC;
                        let addr = self.curr_instr.operand_addr;
                        self.read_idle(pc);
                        self.PC = addr;
                        if !self.curr_instr.zp_crossed
                        {
                            // no page crossing - finish instruction after only 3 cycle
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    1 => {
                        if self.ba_low { return false; }
                        let pc = self.PC;
                        self.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                    },
                    _ => panic!("Wrong number of cycles: {} {} ", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::BMI => {
                match self.curr_instr.cycles_to_run
                {
                    3 => {
                        if self.ba_low { return false; }
                        if self.get_status_flag(StatusFlag::Negative)
                        {
                            let addr = self.curr_instr.operand_addr;
                            let pc = self.PC;
                            self.curr_instr.zp_crossed = (addr >> 8) != (pc >> 8);
                        }
                        else
                        {
                            // no branching - finish instruction after only 2 cycles
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    2 => {
                        if !self.curr_instr.zp_crossed
                        {
                            self.first_irq_cycle += 1;
                            self.first_nmi_cycle += 1;
                        }
                        if self.ba_low { return false; }
                        
                        let pc = self.PC;
                        let addr = self.curr_instr.operand_addr;
                        self.read_idle(pc);
                        self.PC = addr;

                        if !self.curr_instr.zp_crossed
                        {
                            // no page crossing - finish instruction after only 3 cycle
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    1 => {
                        if self.ba_low { return false; }
                        let pc = self.PC;
                        self.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                    },
                    _ => panic!("Wrong number of cycles: {} {} ", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::BPL => {
                match self.curr_instr.cycles_to_run
                {
                    3 => {
                        if self.ba_low { return false; }
                        if !self.get_status_flag(StatusFlag::Negative)
                        {
                            let addr = self.curr_instr.operand_addr;
                            let pc = self.PC;
                            self.curr_instr.zp_crossed = (addr >> 8) != (pc >> 8);
                        }
                        else
                        {
                            // no branching - finish instruction after only 2 cycles
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    2 => {
                        if !self.curr_instr.zp_crossed
                        {
                            self.first_irq_cycle += 1;
                            self.first_nmi_cycle += 1;
                        }
                        if self.ba_low { return false; }
                        
                        let pc = self.PC;
                        let addr = self.curr_instr.operand_addr;
                        self.read_idle(pc);
                        self.PC = addr;

                        if !self.curr_instr.zp_crossed
                        {
                            // no page crossing - finish instruction after only 3 cycle
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    1 => {
                        if self.ba_low { return false; }
                        let pc = self.PC;
                        self.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                    },
                    _ => panic!("Wrong number of cycles: {} {} ", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::BVC => {
                match self.curr_instr.cycles_to_run
                {
                    3 => {
                        if self.ba_low { return false; }
                        if !self.get_status_flag(StatusFlag::Overflow)
                        {
                            let addr = self.curr_instr.operand_addr;
                            let pc = self.PC;
                            self.curr_instr.zp_crossed = (addr >> 8) != (pc >> 8);
                        }
                        else
                        {
                            // no branching - finish instruction after only 2 cycles
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    2 => {
                        if !self.curr_instr.zp_crossed
                        {
                            self.first_irq_cycle += 1;
                            self.first_nmi_cycle += 1;
                        }
                        if self.ba_low { return false; }
                        
                        let pc = self.PC;
                        let addr = self.curr_instr.operand_addr;
                        self.read_idle(pc);
                        self.PC = addr;

                        if !self.curr_instr.zp_crossed
                        {
                            // no page crossing - finish instruction after only 3 cycle
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    1 => {
                        if self.ba_low { return false; }
                        let pc = self.PC;
                        self.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                    },
                    _ => panic!("Wrong number of cycles: {} {} ", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::BVS => {
                match self.curr_instr.cycles_to_run
                {
                    3 => {
                        if self.ba_low { return false; }
                        if self.get_status_flag(StatusFlag::Overflow)
                        {
                            let addr = self.curr_instr.operand_addr;
                            let pc = self.PC;
                            self.curr_instr.zp_crossed = (addr >> 8) != (pc >> 8);
                        }
                        else
                        {
                            // no branching - finish instruction after only 2 cycles
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    2 => {
                        if !self.curr_instr.zp_crossed
                        {
                            self.first_irq_cycle += 1;
                            self.first_nmi_cycle += 1;
                        }
                        if self.ba_low { return false; }
                        
                        let pc = self.PC;
                        let addr = self.curr_instr.operand_addr;
                        self.read_idle(pc);
                        self.PC = addr;

                        if !self.curr_instr.zp_crossed
                        {
                            // no page crossing - finish instruction after only 3 cycle
                            self.curr_instr.cycles_to_run = 1;
                        }
                    },
                    1 => {
                        if self.ba_low { return false; }
                        let pc = self.PC;
                        self.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                    },
                    _ => panic!("Wrong number of cycles: {} {} ", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::CLC => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.set_status_flag(StatusFlag::Carry, false);
            },
            Op::CLD => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.set_status_flag(StatusFlag::DecimalMode, false);
            },
            Op::CLI => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.set_status_flag(StatusFlag::InterruptDisable, false);
            },
            Op::CLV => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.set_status_flag(StatusFlag::Overflow, false);
            },
            Op::SEC => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.set_status_flag(StatusFlag::Carry, true);
            },
            Op::SED => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.set_status_flag(StatusFlag::DecimalMode, true);
            },
            Op::SEI => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
                self.set_status_flag(StatusFlag::InterruptDisable, true);
            },
            Op::BRK => { // TODO: is this ok? do we have to break down new PC value to 2 cycles? read_word ok here?
                match self.curr_instr.cycles_to_run
                {
                    6 => {
                        if self.ba_low { return false; }
                        let pc = self.PC + 0x0001;
                        self.read_idle(pc);
                    },
                    5 => {
                        let pc = self.PC + 0x0001;
                        self.push_byte(((pc >> 8) & 0xFF) as u8);
                    },
                    4 => {
                        let pc = self.PC + 0x0001;
                        self.push_byte((pc & 0xFF) as u8);
                    },
                    3 => {
                        self.set_status_flag(StatusFlag::Break, true);
                        let p = self.P;
                        self.push_byte(p);
                        self.set_status_flag(StatusFlag::InterruptDisable, true);
                        if self.nmi
                        {
                            self.nmi_cycles_left = 7;
                            self.state = CPUState::ProcessNMI;
                        }
                    },
                    2 => {
                        self.first_nmi_cycle += 1; // delay NMI
                    },
                    1  => {
                        //println!("Received BRK instruction at ${:04X}", self.PC-1);
                        self.PC = self.read_word_le(IRQ_VECTOR);
                    },
                    _ => panic!("Wrong number of cycles: {} {} ", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            Op::NOP => {
                if self.ba_low { return false; }
                let pc = self.PC;
                self.read_idle(pc);
            },
            Op::RTI => { // TODO is this ok?
                if self.ba_low { return false; }

                match self.curr_instr.cycles_to_run
                {
                    5 => {
                        let pc = self.PC;
                        self.read_idle(pc);
                    },
                    4 => {
                        let sp = self.SP as u16;
                        self.read_idle(sp + 1);
                    },
                    3 => {
                        let p = self.pop_byte();
                        self.P = p;
                    },
                    2 => {
                        let pc_lo = self.pop_byte() as u16;
                        self.PC = pc_lo;
                    },
                    1  => {
                        let pc_hi = self.pop_byte() as u16;
                        self.PC |= pc_hi << 8;
                    },
                    _ => panic!("Wrong number of cycles: {} {} ", self.curr_instr, self.curr_instr.cycles_to_run)
                }
            },
            // forbidden ops
            Op::HLT => {
                panic!("Received HLT instruction at ${:04X}", self.PC-1);
            },
            Op::SLO => {
                let mut v = self.curr_instr.rmw_buffer;
                let nc = (v & 0x80) != 0;
                self.set_status_flag(StatusFlag::Carry, nc);
                v <<= 1;
                self.set_operand(v);
                let na = self.A | v;
                self.A = na;
                self.set_zn_flags(na);
            },
            Op::ANC => {
                let v = self.get_operand();
                let na = self.A & v;
                self.set_zn_flags(na);
                let n = self.get_status_flag(StatusFlag::Negative);
                self.set_status_flag(StatusFlag::Carry, n);
            },
            Op::RLA => {
                let tmp = self.curr_instr.rmw_buffer & 0x80;
                let c = self.get_status_flag(StatusFlag::Carry);
                let mut v = self.curr_instr.rmw_buffer << 1;
                if c
                {
                    v |= 1;
                }

                self.set_status_flag(StatusFlag::Carry, tmp != 0);
                self.set_operand(v);
                let na = self.A & v;
                self.A = na;
                self.set_zn_flags(na);
            },
            Op::SRE => {
                let mut v = self.curr_instr.rmw_buffer;
                let nc = (v & 0x01) != 0;
                self.set_status_flag(StatusFlag::Carry, nc);
                v >>= 1;
                self.set_operand(v);
                let na = self.A ^ v;
                self.A = na;
                self.set_zn_flags(na);
            },
            Op::RRA => {
                let mut v = self.curr_instr.rmw_buffer;
                let tmp = v & 0x01;
                let c = self.get_status_flag(StatusFlag::Carry);
                v >>= 1;
                if c
                {
                    v |= 0x80;
                }
                self.set_status_flag(StatusFlag::Carry, tmp != 0);

                // todo: copy from adc
                if self.get_status_flag(StatusFlag::DecimalMode)
                {
                    let mut lo = (Wrapping((self.A as u16) & 0xF) + Wrapping((v as u16) & 0xF)).0;
                    if  c { lo = (Wrapping(lo) + Wrapping(1)).0; }
                    if lo > 9 { lo = (Wrapping(lo) + Wrapping(6)).0; }

                    let mut hi = (Wrapping((self.A as u16) >> 4) + Wrapping((v as u16) >> 4)).0;
                    if lo > 0xF { hi = (Wrapping(hi) + Wrapping(1)).0; }

                    let is_overflow = ((((hi << 4) ^ (self.A as u16)) & 0x80) != 0) && (((self.A ^ v) & 0x80) == 0);
                    let mut is_zero = (Wrapping(self.A as u16) + Wrapping(v as u16)).0;
                    if c  { is_zero = (Wrapping(is_zero) + Wrapping(1)).0; }
                    
                    self.set_status_flag(StatusFlag::Negative, (hi << 4) != 0); // TODO: is this ok?              
                    self.set_status_flag(StatusFlag::Overflow, is_overflow);
                    self.set_status_flag(StatusFlag::Zero,     is_zero == 0);

                    if hi > 9 { hi = (Wrapping(hi) + Wrapping(6)).0; }
                    self.set_status_flag(StatusFlag::Carry, hi > 0xF);
                    self.A = ((hi << 4) | (lo & 0xF)) as u8;
                }
                else
                {
                    // TODO: should operation wrap automatically here?
                    let mut res: u16 = (Wrapping(self.A as u16) + Wrapping(v as u16)).0;
                    if c
                    {
                        res = (Wrapping(res) + Wrapping(0x0001)).0;
                    }
                    self.set_status_flag(StatusFlag::Carry, (res & 0x0100) != 0);
                    let res = res as u8;
                    let is_overflow = (self.A ^ v) & 0x80 == 0 && (self.A ^ res) & 0x80 == 0x80;
                    self.set_status_flag(StatusFlag::Overflow, is_overflow);
                    self.A = res;
                    self.set_zn_flags(res);
                }
                
            },
            Op::SAX => {
                let v = self.A & self.X;
                self.set_operand(v);
            },
            Op::AHX => {
                let addr = self.curr_instr.operand_addr;
                let addr_hi = self.curr_instr.index_addr as u8;
                let y = self.Y;
                self.write_byte(addr, y & (addr_hi + 1));
            },
            Op::TAS => {
                let addr = self.curr_instr.operand_addr;
                let addr_hi = self.curr_instr.index_addr as u8;
                let a = self.A;
                let x = self.X;
                self.SP = a & x;
                self.write_byte(addr, (a & x) & (addr_hi + 1));
            },
            Op::SHY => {
                let addr = self.curr_instr.operand_addr;
                let addr_hi = self.curr_instr.index_addr as u8;
                let a = self.A;
                let x = self.X;
                self.write_byte(addr, a & x & (addr_hi + 1));
            },
            Op::SHX => {
                let addr = self.curr_instr.operand_addr;
                let addr_hi = self.curr_instr.index_addr as u8;
                let x = self.X;
                self.write_byte(addr, x & (addr_hi + 1));
            },
            Op::LAX => {
                if self.ba_low { return false; }
                let nv = self.get_operand();
                self.A = nv;
                self.X = nv;
                self.set_zn_flags(nv);
            }, 
            Op::DCP => {
                let v = (Wrapping(self.curr_instr.rmw_buffer) - Wrapping(0x01)).0;
                self.set_operand(v);
                let diff = (Wrapping(self.A) - Wrapping(v)).0;
                self.set_zn_flags(diff);
                self.set_status_flag(StatusFlag::Carry, (diff & 0x0100) == 0);
            },
            Op::ISC => {
                self.curr_instr.rmw_buffer += 1;
                let v = self.curr_instr.rmw_buffer;
                self.set_operand(v);

                // copy of SBC; TODO: common func?
                let mut res: u16 = (Wrapping(self.A as u16) - Wrapping(v as u16)).0;
                if !self.get_status_flag(StatusFlag::Carry)
                {
                    res = (Wrapping(res) - Wrapping(0x0001)).0;
                }
                
                if self.get_status_flag(StatusFlag::DecimalMode)
                {
                    let mut lo = (Wrapping((self.A as u16) & 0xF) - Wrapping((v as u16) & 0xF)).0;
                    let mut hi = (Wrapping((self.A as u16) >> 4) - Wrapping((v as u16) >> 4)).0;

                    if !self.get_status_flag(StatusFlag::Carry)
                    {
                        lo = (Wrapping(lo) - Wrapping(1)).0;
                    }
                    
                    if (lo & 0x10) != 0
                    {
                        lo = (Wrapping(lo) - Wrapping(6)).0;
                        hi = (Wrapping(hi) - Wrapping(1)).0;
                    }

                    if (hi & 0x10) != 0 { hi = (Wrapping(hi) - Wrapping(6)).0; }

                    self.set_status_flag(StatusFlag::Carry, (res & 0x0100) == 0);
                    let res = res as u8;
                    let is_overflow = (self.A ^ res) & 0x80 != 0 && (self.A ^ v) & 0x80 == 0x80;
                    self.set_status_flag(StatusFlag::Overflow, is_overflow);
                    self.set_zn_flags(res);

                    self.A = ((hi << 4) | (lo & 0xF)) as u8;
                }
                else
                {
                    // TODO: should operation wrap automatically here?
                    self.set_status_flag(StatusFlag::Carry, (res & 0x0100) == 0);
                    let res = res as u8;
                    let is_overflow = (self.A ^ res) & 0x80 != 0 && (self.A ^ v) & 0x80 == 0x80;
                    self.set_status_flag(StatusFlag::Overflow, is_overflow);
                    self.A = res;
                    self.set_zn_flags(res);
                }
            },
            _ => panic!("Unknown instruction: {} at ${:04X}", self.curr_instr, self.PC)
        }

        self.curr_instr.cycles_to_run -= 1;
        // instruction finished execution
        self.curr_instr.cycles_to_run == 0
    }
}
