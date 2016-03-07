// The CPU
use c64::cia;
use c64::memory;
use c64::opcodes;
use c64::sid;
use c64::vic;
use std::cell::RefCell;
use std::rc::Rc;
use utils;

pub type CPUShared = Rc<RefCell<CPU>>;

pub const NMI_VECTOR:   u16 = 0xFFFA;
pub const RESET_VECTOR: u16 = 0xFFFC;
pub const IRQ_VECTOR:   u16 = 0xFFFE;


// status flags for P register
pub enum StatusFlag {
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
pub enum Callback {
    None,
    TriggerVICIrq,
    ClearVICIrq,
    TriggerCIAIrq,
    ClearCIAIrq,
    TriggerNMI,
    ClearNMI
}

pub enum CPUState {
    FetchOp,
    FetchOperandAddr,
    PerformRMW,
    ProcessIRQ,
    ProcessNMI,
    ExecuteOp
}


pub struct CPU {
    pub pc: u16, // program counter
    pub sp: u8,  // stack pointer
    pub p:  u8,  // processor status
    pub a:  u8,  // accumulator
    pub x:  u8,  // index register
    pub y:  u8,  // index register
    pub mem_ref:  Option<memory::MemShared>, // reference to shared system memory
    pub vic_ref:  Option<vic::VICShared>,
    pub cia1_ref: Option<cia::CIAShared>,
    pub cia2_ref: Option<cia::CIAShared>,
    pub sid_ref:  Option<sid::SIDShared>,
    pub instruction: opcodes::Instruction,
    pub ba_low:  bool,  // is BA low?
    pub cia_irq: bool,
    pub vic_irq: bool,
    pub irq_cycles_left: u8,
    pub nmi_cycles_left: u8,
    pub first_nmi_cycle: u32,
    pub first_irq_cycle: u32,
    pub state: CPUState,
    pub nmi: bool,
    pub debug_instr: bool,
    pub prev_pc: u16, // previous program counter - used for debugging
    pub op_debugger: utils::OpDebugger,
    dfff_byte: u8
}

impl CPU {
    pub fn new_shared() -> CPUShared {
        Rc::new(RefCell::new(CPU {
            pc: 0,
            sp: 0xFF,
            p:  0,
            a:  0,
            x:  0,
            y:  0,
            mem_ref:  None,
            vic_ref:  None,
            cia1_ref: None,
            cia2_ref: None,
            sid_ref:  None,
            ba_low:  false,
            cia_irq: false,
            vic_irq: false,
            irq_cycles_left: 0,
            nmi_cycles_left: 0,
            first_nmi_cycle: 0,
            first_irq_cycle: 0,
            state: CPUState::FetchOp,
            instruction: opcodes::Instruction::new(),
            nmi: false,
            debug_instr: false,
            prev_pc: 0,
            op_debugger: utils::OpDebugger::new(),
            dfff_byte: 0x55
        }))
    }


    pub fn set_references(&mut self, memref: memory::MemShared, vicref: vic::VICShared, cia1ref: cia::CIAShared, cia2ref: cia::CIAShared, sidref: sid::SIDShared) {
        self.mem_ref = Some(memref);
        self.vic_ref = Some(vicref);
        self.cia1_ref = Some(cia1ref);
        self.cia2_ref = Some(cia2ref);
        self.sid_ref  = Some(sidref);
    }    
    

    pub fn set_status_flag(&mut self, flag: StatusFlag, value: bool) {
        if value { self.p |=   flag as u8;  }
        else     { self.p &= !(flag as u8); }
    }


    pub fn get_status_flag(&mut self, flag: StatusFlag) -> bool {
        self.p & flag as u8 != 0x00
    }


    // these flags will be set in tandem quite often
    pub fn set_zn_flags(&mut self, value: u8) {
        self.set_status_flag(StatusFlag::Zero, value == 0x00);
        self.set_status_flag(StatusFlag::Negative, (value as i8) < 0);
    }
    

    pub fn reset(&mut self) {
        let pc = self.read_word_le(RESET_VECTOR);
        self.pc = pc;

        // I'm only doing this to avoid dead code warning :)
        self.set_status_flag(StatusFlag::Unused, false);
    }


    pub fn update(&mut self, c64_cycle_cnt: u32) {
        // check for irq and nmi
        match self.state {
            CPUState::FetchOp => {
                if self.nmi && self.nmi_cycles_left == 0 && (c64_cycle_cnt - (self.first_nmi_cycle as u32) >= 2) {
                    self.nmi_cycles_left = 7;
                    self.state = CPUState::ProcessNMI;
                }
                else if (self.cia_irq || self.vic_irq) && self.irq_cycles_left == 0 && !self.get_status_flag(StatusFlag::InterruptDisable) && (c64_cycle_cnt - (self.first_irq_cycle as u32) >= 2) {
                    self.irq_cycles_left = 7;
                    self.state = CPUState::ProcessIRQ;
                }
            },
            _ => {}
        }
        
        match self.state {
            CPUState::FetchOp => {
                if self.ba_low { return; }
                let next_op = self.next_byte();
                match opcodes::get_instruction(next_op) {
                    Some((opcode, total_cycles, is_rmw, addr_mode)) => {
                        self.instruction.opcode = opcode;
                        self.instruction.addr_mode = addr_mode;
                        self.instruction.is_rmw = is_rmw;
                        self.instruction.calculate_cycles(total_cycles, is_rmw);
                        if self.debug_instr { utils::debug_instruction(next_op, self); }
                    }
                    None => panic!("Can't fetch instruction")
                }

                // jump straight to op execution unless operand address needs to be fetched
                match self.instruction.addr_mode {
                    opcodes::AddrMode::Implied     => self.state = CPUState::ExecuteOp,
                    opcodes::AddrMode::Accumulator => self.state = CPUState::ExecuteOp,
                    opcodes::AddrMode::Immediate   => self.state = CPUState::ExecuteOp,
                    opcodes::AddrMode::Relative    => {
                        // TODO: inc PC only during op execution?
                        let base = (self.pc + 1) as i16;
                        let offset = self.next_byte() as i8;
                        self.instruction.operand_addr = (base + offset as i16) as u16;
                        self.state = CPUState::ExecuteOp;
                    },
                    _ => self.state = CPUState::FetchOperandAddr,
                };
            },
            CPUState::FetchOperandAddr => {
                if self.ba_low { return; }
                if opcodes::fetch_operand_addr(self) {
                    if self.instruction.is_rmw {
                        self.state = CPUState::PerformRMW;
                    }
                    else {
                        self.state = CPUState::ExecuteOp;
                    }
                }

                // TODO: odd case? Some instructions can be executed immediately after operand fetch
                if self.instruction.cycles_to_run == 0 && self.instruction.cycles_to_fetch == 0 {
                    //panic!("Not sure if this should happen - reinvestigate");
                    opcodes::run(self);
                    self.state = CPUState::FetchOp;
                }
            }
            CPUState::ProcessIRQ => {
                if self.process_irq(false) {
                    self.cia_irq = false;
                    self.vic_irq = false;
                    self.state = CPUState::FetchOp;
                }
            },
            CPUState::ProcessNMI => {
                if self.process_irq(true) {
                    self.nmi = false;
                    self.state = CPUState::FetchOp;
                }
            },
            CPUState::PerformRMW => {
                match self.instruction.cycles_to_rmw {
                    2 => {
                        if self.ba_low { return; }
                        let addr = self.instruction.operand_addr;
                        self.instruction.rmw_buffer = self.read_byte(addr);
                    },
                    1 => {
                        let addr = self.instruction.operand_addr;
                        let val = self.instruction.rmw_buffer;
                        self.write_byte(addr, val);
                        self.state = CPUState::ExecuteOp;
                    },
                     _ => panic!("Too many cycles in RMW stage! ({}) ", self.instruction.cycles_to_rmw)
                }

                self.instruction.cycles_to_rmw -= 1;
            },
            CPUState::ExecuteOp => {
                if opcodes::run(self) {
                    self.state = CPUState::FetchOp;
                }
            }
        }
    }


    pub fn next_byte(&mut self) -> u8 {
        let pc = self.pc;
        let op = self.read_byte(pc);
        self.pc += 1;
        op
    }


    // stack memory: $0100 - $01FF (256 byes)
    pub fn push_byte(&mut self, value: u8) {
        self.sp -= 0x01;
        let new_sp = (self.sp + 0x01) as u16;
        self.write_byte(0x0100 + new_sp, value);
    }


    pub fn pop_byte(&mut self) -> u8 {
        let addr = 0x0100 + (self.sp + 0x01) as u16;
        let value = self.read_byte(addr);
        self.sp += 0x01;
        value
    }


    pub fn push_word(&mut self, value: u16) {
        self.push_byte(((value >> 8) & 0xFF) as u8);
        self.push_byte((value & 0xFF) as u8);
    }


    pub fn write_byte(&mut self, addr: u16, value: u8) -> bool {
        let mut on_write = Callback::None;
        let mut mem_write_ok = true;
        let io_enabled = as_ref!(self.mem_ref).io_on;

        if io_enabled {
            match addr {
 /*   VIC-II  */ 0xD000...0xD3FF => as_mut!(self.vic_ref).write_register(addr, value, &mut on_write),
 /*    SID    */ 0xD400...0xD7FF => as_mut!(self.sid_ref).write_register(addr, value),
 /* color RAM */ 0xD800...0xDBFF => mem_write_ok = as_mut!(self.mem_ref).write_byte(addr, value & 0x0F),
 /*    CIA1   */ 0xDC00...0xDCFF => as_mut!(self.cia1_ref).write_register(addr, value, &mut on_write),
 /*    CIA2   */ 0xDD00...0xDDFF => as_mut!(self.cia2_ref).write_register(addr, value, &mut on_write),
                 _               => mem_write_ok = as_mut!(self.mem_ref).write_byte(addr, value),
            }
        }
        else {
            mem_write_ok = as_mut!(self.mem_ref).write_byte(addr, value);
        }

        // on VIC/CIA register write perform necessary action on the CPU
        match on_write {
            Callback::TriggerVICIrq => self.set_vic_irq(true),
            Callback::ClearVICIrq   => self.set_vic_irq(false),
            Callback::TriggerCIAIrq => self.set_cia_irq(true),
            Callback::ClearCIAIrq   => self.set_cia_irq(false),
            Callback::TriggerNMI    => self.set_nmi(true),
            Callback::ClearNMI      => self.set_nmi(false),
            _ => (),
        }

        mem_write_ok
    }
    

    pub fn read_byte(&mut self, addr: u16) -> u8 {
        let byte: u8;
        let mut on_read = Callback::None;
        let io_enabled = as_ref!(self.mem_ref).io_on;

        if io_enabled {
            match addr {
   /*  VIC-II   */ 0xD000...0xD3FF => byte = as_mut!(self.vic_ref).read_register(addr),
   /*   SID     */ 0xD400...0xD7FF => byte = as_mut!(self.sid_ref).read_register(addr),
   /* color RAM */ 0xD800...0xDBFF => byte = (as_ref!(self.mem_ref).read_byte(addr) & 0x0F) | (as_ref!(self.vic_ref).last_byte & 0xF0),
   /*   CIA1    */ 0xDC00...0xDCFF => byte = as_mut!(self.cia1_ref).read_register(addr, &mut on_read),
   /*   CIA2    */ 0xDD00...0xDDFF => byte = as_mut!(self.cia2_ref).read_register(addr, &mut on_read),
                   0xDF00...0xDF9F => byte = as_ref!(self.vic_ref).last_byte,
                   0xDFFF => {
                       self.dfff_byte = !self.dfff_byte;
                       byte = self.dfff_byte;
                   },
                   _ => byte = as_mut!(self.mem_ref).read_byte(addr)
            }
        }
        else {
            byte = as_mut!(self.mem_ref).read_byte(addr);
        }

        match on_read {
            Callback::TriggerCIAIrq => self.set_cia_irq(true),
            Callback::ClearCIAIrq   => self.set_cia_irq(false),
            Callback::TriggerNMI    => self.set_nmi(true),
            Callback::ClearNMI      => self.set_nmi(false),
            _ => (),
        }

        byte
    }


    pub fn read_word_le(&self, addr: u16) -> u16 {
        as_ref!(self.mem_ref).read_word_le(addr)
    }


    pub fn set_vic_irq(&mut self, val: bool) {
        self.vic_irq = val;
    }


    pub fn set_nmi(&mut self, val: bool) {
        self.nmi = val;
    }


    pub fn set_cia_irq(&mut self, val: bool) {
        self.cia_irq = val;
    }
    

    pub fn get_operand(&mut self) -> u8 {
        // RMW instruction store pre-fetched operand value in internal buffer
        if self.instruction.is_rmw {
            return self.instruction.rmw_buffer;
        }

        let val = match self.instruction.addr_mode {
            opcodes::AddrMode::Implied     => panic!("Can't get operand value!"),
            opcodes::AddrMode::Accumulator => self.a,
            opcodes::AddrMode::Immediate   => self.next_byte(),
            _ => {
                let addr = self.instruction.operand_addr;
                self.read_byte(addr)
            }
        };

        val
    }


    pub fn set_operand(&mut self, val: u8) {
        match self.instruction.addr_mode {
            opcodes::AddrMode::Implied     => panic!("Can't set implied operand value!"),
            opcodes::AddrMode::Accumulator => self.a = val,
            opcodes::AddrMode::Immediate   => panic!("Can't set immediate operand value!"),
            opcodes::AddrMode::Relative    => panic!("Can't set relative operand value!"),
            _ => {
                let addr = self.instruction.operand_addr;
                let _ = self.write_byte(addr, val);
            }
        }
    }


    // perform add with carry
    pub fn adc(&mut self, value: u8) {
        let c = self.get_status_flag(StatusFlag::Carry);
        let a = self.a as u16;
        let v = value as u16;

        if self.get_status_flag(StatusFlag::DecimalMode) {
            let mut lo = (a & 0x0F).wrapping_add(v & 0x0F);

            if  c {
                lo = lo.wrapping_add(0x01);
            }
            
            if lo > 9 {
                lo = lo.wrapping_add(0x06);
            }

            let mut hi = (a >> 4).wrapping_add(v >> 4);
            if lo > 0x0F {
                hi = hi.wrapping_add(0x01);
            }

            let is_overflow = ((((hi << 4) ^ a) & 0x80) != 0) && (((a ^ v) & 0x80) == 0);
            let mut is_zero = a.wrapping_add(v);

            if c  {
                is_zero = is_zero.wrapping_add(0x01);
            }
            
            self.set_status_flag(StatusFlag::Negative, (hi << 4) != 0); // TODO: is this ok?              
            self.set_status_flag(StatusFlag::Overflow, is_overflow);
            self.set_status_flag(StatusFlag::Zero,     is_zero == 0);

            if hi > 9 {
                hi = hi.wrapping_add(0x06);
            }
            
            self.set_status_flag(StatusFlag::Carry, hi > 0xF);
            self.a = ((hi << 4) | (lo & 0xF)) as u8;
        }
        else {
            // TODO: should operation wrap automatically here?
            let mut res = a.wrapping_add(v);
            
            if c
            {
                res = res.wrapping_add(0x1);
            }
            
            self.set_status_flag(StatusFlag::Carry, (res & 0x0100) != 0);
            let is_overflow = (a ^ v) & 0x80 == 0 && (a ^ res) & 0x80 == 0x80;
            self.set_status_flag(StatusFlag::Overflow, is_overflow);
            self.a = res as u8;
            self.set_zn_flags(res as u8);
        }
    }


    // perform substraction with carry
    pub fn sbc(&mut self, value: u8) {
        let a = self.a as u16;
        let v = value as u16;
        let mut res: u16 = a.wrapping_sub(v);
        
        if !self.get_status_flag(StatusFlag::Carry) {
            res = res.wrapping_sub(0x0001);
        }
        
        if self.get_status_flag(StatusFlag::DecimalMode) {
            let mut lo = (a & 0x0F).wrapping_sub(v & 0x0F);
            let mut hi = (a >> 4).wrapping_sub(v >> 4);

            if !self.get_status_flag(StatusFlag::Carry) {
                lo = lo.wrapping_sub(0x01);
            }
            
            if (lo & 0x10) != 0 {
                lo = lo.wrapping_sub(0x06);
                hi = hi.wrapping_sub(0x01);
            }

            if (hi & 0x10) != 0 {
                hi = hi.wrapping_sub(0x06);
            }

            let is_overflow = (a ^ res) & 0x80 != 0 && (a ^ v) & 0x80 == 0x80;
            
            self.set_status_flag(StatusFlag::Carry, (res & 0x0100) == 0);
            self.set_status_flag(StatusFlag::Overflow, is_overflow);
            self.set_zn_flags(res as u8);

            self.a = ((hi << 4) | (lo & 0xF)) as u8;
        }
        else {
            // TODO: should operation wrap automatically here?
            self.set_status_flag(StatusFlag::Carry, (res & 0x0100) == 0);
            let is_overflow = (a ^ res) & 0x80 != 0 && (a ^ v) & 0x80 == 0x80;
            self.set_status_flag(StatusFlag::Overflow, is_overflow);
            self.a = res as u8;
            self.set_zn_flags(res as u8);
        }
    }


    // perform a branch
    pub fn branch(&mut self, flag_condition: bool, cycle: u8) -> bool {
        match cycle {
            3 => {
                if self.ba_low { return false; }
                if flag_condition {
                    let addr = self.instruction.operand_addr;
                    let pc = self.pc;
                    self.instruction.zp_crossed = (addr >> 8) != (pc >> 8);
                }
                else {
                    // no branching - finish instruction after only 2 cycles
                    self.instruction.cycles_to_run = 1;
                }
            },
            2 => {
                if !self.instruction.zp_crossed {
                    self.first_irq_cycle += 1;
                    self.first_nmi_cycle += 1;
                }
                if self.ba_low { return false; }
                
                let addr = self.instruction.operand_addr;
                self.pc = addr;

                if !self.instruction.zp_crossed {
                    // no page crossing - finish instruction after only 3 cycle
                    self.instruction.cycles_to_run = 1;
                }
            },
            1 => {
                if self.ba_low { return false; }
            },
            _ => panic!("Wrong number of branching cycles"),
        }

        true
    }


    // *** private functions *** //

    fn process_irq(&mut self, is_nmi: bool) -> bool {
        let new_pc    = if is_nmi { NMI_VECTOR } else { IRQ_VECTOR };
        let cycle_cnt = if is_nmi { self.nmi_cycles_left } else { self.irq_cycles_left };
        
        match cycle_cnt {
            7 | 6 => {
                if self.ba_low { return false; }
            },
            5 => {
                let pc_hi = (self.pc >> 8) as u8;
                self.push_byte(pc_hi);
            },
            4 => {
                let pc_lo = self.pc as u8;
                self.push_byte(pc_lo);
            },
            3 => {
                self.set_status_flag(StatusFlag::Break, false);
                let curr_p = self.p;
                self.push_byte(curr_p);
                self.set_status_flag(StatusFlag::InterruptDisable, true);
            },
            2 => {
                if self.ba_low { return false; } // TODO: is reading whole word ok in cycle 1?
            },
            1 => {
                if self.ba_low { return false; }
                self.pc = as_ref!(self.mem_ref).read_word_le(new_pc);
            }
            _ => panic!("Invalid IRQ/NMI cycle")
        }

        if is_nmi {
            self.nmi_cycles_left -= 1;
            self.nmi_cycles_left == 0
        }
        else {
            self.irq_cycles_left -= 1;
            self.irq_cycles_left == 0
        }
    }
}
