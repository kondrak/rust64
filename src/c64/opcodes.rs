// opcode enumeration suffix: // addressing mode:
// imm = #$00                 // immediate 
// zp = $00                   // zero page
// zpx = $00,X                // zero page with X
// zpy = $00,Y                // zero page with Y
// izx = ($00,X)              // indexed indirect (X)
// izy = ($00),Y              // indirect indexed (Y)
// abs = $0000                // absolute
// abx = $0000,X              // absolute indexed with X
// aby = $0000,Y              // absolute indexed with Y
// ind = ($0000)              // indirect
// rel = $0000                // relative to PC/IP

#![allow(non_camel_case_types)]
use c64::cpu;
use std::fmt;
use std::num::Wrapping;

pub enum AddrMode
{
    Implied,
    Accumulator,
    Immediate,
    Absolute,
    AbsoluteIndexedX(bool),
    AbsoluteIndexedY(bool),
    Zeropage,
    ZeropageIndexedX,
    ZeropageIndexedY,
    Relative,
    Indirect,
    IndexedIndirectX,
    IndirectIndexedY(bool)
}

pub enum Op {
    // Load/store
    LDA, LDX, LDY,
    STA, STX, STY,
    // Register transfers
    TAX, TAY, TXA,
    TYA,
    // Stack operations
    TSX, TXS, PHA,
    PHP, PLA, PLP,
    // Logical
    AND, EOR, ORA,
    BIT,
    // Arithmetic
    ADC, SBC, CMP,
    CPX, CPY,
    // Inc/Dec
    INC, INX, INY,
    DEC, DEX, DEY,
    // Shifts
    ASL, LSR, ROL,
    ROR,
    // Jump calls
    JMP, JSR, RTS,
    // Branches
    BCC, BCS, BEQ,
    BMI, BNE, BPL,
    BVC, BVS,
    // Status flag changes
    CLC, CLD, CLI,
    CLV, SEC, SED,
    SEI,
    // System functions
    BRK, NOP, RTI,
    // forbidden/undocumented
    HLT, SLO, ANC,
    RLA, SRE, RRA,
    ALR, SAX, XAA,
    AHX, TAS, SHY,
    SHX, ARR, LAX,
    LAS, DCP, AXS,
    ISC
}

pub struct Instruction
{
    pub addr_mode: AddrMode,
    pub opcode: Op,
    pub operand_addr: u16,  // operand address for other modes
    pub index_addr: u16,    // additional address storage for indirect and indexed addressing modes
    pub cycles_to_fetch: u8, // how many cycles to fetch the operand?
    pub cycles_to_run: u8, // how many cycles to execute the operation?
    pub cycles_to_rmw: u8, // how many cycles for additional 2 steps of read-move-write?
    pub is_rmw: bool,
    pub rmw_buffer: u8,   // data read and stored during rmw stage and used as operand addr during execution for rmw instruction
    pub zp_crossed: bool, // zero page crossed?
}

impl Instruction
{
    pub fn new() -> Instruction
    {
        Instruction {
            opcode: Op::BRK,
            addr_mode: AddrMode::Implied,
            operand_addr: 0,
            index_addr: 0,
            cycles_to_fetch: 0,
            cycles_to_run: 0,
            cycles_to_rmw: 0,
            is_rmw: false,
            rmw_buffer: 0,
            zp_crossed: false,
        }
    }

    pub fn calculate_cycles(&mut self, total_cycles: u8, is_rmw: bool)
    {
        match self.addr_mode {
            AddrMode::Absolute => self.cycles_to_fetch = 2,
            AddrMode::Zeropage => self.cycles_to_fetch = 1,
            AddrMode::Indirect => self.cycles_to_fetch = 2,
            AddrMode::ZeropageIndexedX => self.cycles_to_fetch = 2,
            AddrMode::ZeropageIndexedY => self.cycles_to_fetch = 2,
            AddrMode::IndexedIndirectX => self.cycles_to_fetch = 4,
            AddrMode::AbsoluteIndexedX(..) => self.cycles_to_fetch = 3,
            AddrMode::AbsoluteIndexedY(..) => self.cycles_to_fetch = 3,
            AddrMode::IndirectIndexedY(..) => self.cycles_to_fetch = 4,
            _ => {}
        }        
        
        self.cycles_to_rmw = if is_rmw { 2 } else { 0 };
        self.cycles_to_run = total_cycles - self.cycles_to_fetch - self.cycles_to_rmw;

        // subtract 1 to account for op fetching which already took 1 cycle
        self.cycles_to_run -= 1;
    }
}

// debug display for opcodes
impl fmt::Display for Instruction
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let op_name = match self.opcode {
            Op::LDA => "LDA", Op::LDX => "LDX", Op::LDY => "LDY", Op::STA => "STA",
            Op::STX => "STX", Op::STY => "STY", Op::TAX => "TAX", Op::TAY => "TAY",
            Op::TXA => "TXA", Op::TYA => "TYA", Op::TSX => "TSX", Op::TXS => "TXS",
            Op::PHA => "PHA", Op::PHP => "PHP", Op::PLA => "PLA", Op::PLP => "PLP",
            Op::AND => "AND", Op::EOR => "EOR", Op::ORA => "ORA", Op::BIT => "BIT",
            Op::ADC => "ADC", Op::SBC => "SBC", Op::CMP => "CMP", Op::CPX => "CPX",
            Op::CPY => "CPY", Op::INC => "INC", Op::INX => "INX", Op::INY => "INY",
            Op::DEC => "DEC", Op::DEX => "DEX", Op::DEY => "DEY", Op::ASL => "ASL",
            Op::LSR => "LSR", Op::ROL => "ROL", Op::ROR => "ROR", Op::JMP => "JMP",
            Op::JSR => "JSR", Op::RTS => "RTS", Op::BCC => "BCC", Op::BCS => "BCS",
            Op::BEQ => "BEQ", Op::BMI => "BMI", Op::BNE => "BNE", Op::BPL => "BPL",
            Op::BVC => "BVC", Op::BVS => "BVS", Op::CLC => "CLC", Op::CLD => "CLD",
            Op::CLI => "CLI", Op::CLV => "CLV", Op::SEC => "SEC", Op::SED => "SED",
            Op::SEI => "SEI", Op::BRK => "BRK", Op::NOP => "NOP", Op::RTI => "RTI",
            Op::HLT => "HLT", Op::SLO => "SLO", Op::ANC => "ANC", Op::RLA => "RLA",
            Op::SRE => "SRE", Op::RRA => "RRA", Op::ALR => "ALR", Op::SAX => "SAX",
            Op::XAA => "XAA", Op::AHX => "AHX", Op::TAS => "TAS", Op::SHY => "SHY",
            Op::SHX => "SHX", Op::ARR => "ARR", Op::LAX => "LAX", Op::LAS => "LAS",
            Op::DCP => "DCP", Op::AXS => "AXS", Op::ISC => "ISC",
        };
        
        write!(f, "{}", op_name)
    }
}

pub fn fetch_operand_addr(cpu: &mut cpu::CPU) -> bool
{
    match cpu.instruction.addr_mode
    {
        AddrMode::Absolute => {
            match cpu.instruction.cycles_to_fetch {
                2 => {
                    cpu.instruction.operand_addr = cpu.next_byte() as u16;
                },
                1 => {
                    cpu.instruction.operand_addr = cpu.instruction.operand_addr | ((cpu.next_byte() as u16) << 8);
                },
                _ => panic!("Too many cycles for operand address fetch! ({}) ", cpu.instruction.cycles_to_fetch)
            }
        },
        AddrMode::AbsoluteIndexedX(extra_cycle) => {
            match cpu.instruction.cycles_to_fetch {
                3 => {
                    cpu.instruction.operand_addr = cpu.next_byte() as u16;
                },
                2 => {
                    let addr_lo = cpu.instruction.operand_addr;
                    cpu.instruction.index_addr = cpu.next_byte() as u16;
                    cpu.instruction.operand_addr = ((addr_lo + cpu.X as u16) & 0xFF) | (cpu.instruction.index_addr << 8);
                    // page crossed?
                    cpu.instruction.zp_crossed = addr_lo + (cpu.X as u16) >= 0x100;

                    // if instruction has extra cycle on page crossing and it hasn't happened, we don't get
                    // the extra cycle (finish fetching now)
                    if !cpu.instruction.zp_crossed && extra_cycle
                    {
                        cpu.instruction.cycles_to_fetch = 1;
                    }
                },
                1 => { // if page crossed - add 0x100 to operand address
                    let addr = cpu.instruction.operand_addr;
                    cpu.read_idle(addr);
                    if cpu.instruction.zp_crossed { cpu.instruction.operand_addr += 0x100; }
                },
                _ => panic!("Too many cycles for operand address fetch! ({}) ", cpu.instruction.cycles_to_fetch)
            }
        },
        AddrMode::AbsoluteIndexedY(extra_cycle) => {
            match cpu.instruction.cycles_to_fetch {
                3 => {
                    cpu.instruction.operand_addr = cpu.next_byte() as u16;
                },
                2 => {
                    cpu.instruction.index_addr = cpu.next_byte() as u16;
                    let addr_lo = cpu.instruction.operand_addr;
                    cpu.instruction.operand_addr = ((addr_lo + cpu.Y as u16) & 0xFF) | (cpu.instruction.index_addr << 8);
                    // page crossed?
                    cpu.instruction.zp_crossed = addr_lo + (cpu.Y as u16) >= 0x100;
                    
                    // if instruction has extra cycle on page crossing and it hasn't happened, we don't get
                    // the extra cycle (finish fetching now)
                    if !cpu.instruction.zp_crossed && extra_cycle
                    {
                        cpu.instruction.cycles_to_fetch = 1;
                    }
                },
                1 => { // if page crossed - add 0x100 to operand address
                    let addr = cpu.instruction.operand_addr;
                    cpu.read_idle(addr);
                    if cpu.instruction.zp_crossed { cpu.instruction.operand_addr += 0x100; }
                },
                _ => panic!("Too many cycles for operand address fetch! ({}) ", cpu.instruction.cycles_to_fetch)
            }
        },
        AddrMode::Zeropage => {
            cpu.instruction.operand_addr = cpu.next_byte() as u16;
        },
        AddrMode::ZeropageIndexedX => {
            match cpu.instruction.cycles_to_fetch {
                2 => {
                    cpu.instruction.operand_addr = cpu.next_byte() as u16;
                },
                1 => {
                    let x = cpu.X as u16;
                    let base_addr = cpu.instruction.operand_addr;
                    cpu.read_idle(base_addr);
                    cpu.instruction.operand_addr = ((Wrapping(base_addr) + Wrapping(x)).0 as u16) & 0xFF;
                }
                _ => panic!("Too many cycles for operand address fetch! ({}) ", cpu.instruction.cycles_to_fetch)
            }
        },
        AddrMode::ZeropageIndexedY => {
            match cpu.instruction.cycles_to_fetch {
                2 => {
                    cpu.instruction.operand_addr = cpu.next_byte() as u16;
                },
                1 => {
                    let y = cpu.Y as u16;
                    let base_addr = cpu.instruction.operand_addr;
                    cpu.read_idle(base_addr);
                    cpu.instruction.operand_addr = ((Wrapping(base_addr) + Wrapping(y)).0 as u16) & 0xFF;
                }
                _ => panic!("Too many cycles for operand address fetch! ({}) ", cpu.instruction.cycles_to_fetch)
            }
        },
        AddrMode::IndexedIndirectX => {
            match cpu.instruction.cycles_to_fetch {
                4 => {
                    cpu.instruction.index_addr = cpu.next_byte() as u16;
                },
                3 => {
                    let addr = cpu.instruction.index_addr;
                    cpu.read_idle(addr);
                    cpu.instruction.index_addr = (cpu.instruction.index_addr + cpu.X as u16) & 0xFF;
                },
                2 => {
                    let idx_addr = cpu.instruction.index_addr;
                    cpu.instruction.operand_addr =  cpu.read_byte(idx_addr) as u16;
                },
                1 => {
                    let idx = cpu.instruction.index_addr;
                    let hi = cpu.read_byte((idx + 1) & 0xFF) as u16;
                    cpu.instruction.operand_addr = cpu.instruction.operand_addr | (hi << 8);
                },
                _ => panic!("Too many cycles for operand address fetch! ({}) ", cpu.instruction.cycles_to_fetch)
            }
        },
        AddrMode::IndirectIndexedY(extra_cycle) => {
            match cpu.instruction.cycles_to_fetch {
                4 => {
                    cpu.instruction.index_addr = cpu.next_byte() as u16;
                },
                3 => {
                    let base_addr = cpu.instruction.index_addr;
                    cpu.instruction.operand_addr = cpu.read_byte(base_addr) as u16;
                },
                2 => {
                    let idx = cpu.instruction.index_addr;
                    let opaddr = cpu.instruction.operand_addr;
                    cpu.instruction.index_addr =  cpu.read_byte((idx + 1) & 0xFF ) as u16;
                    cpu.instruction.operand_addr = ((opaddr + cpu.Y as u16) & 0x0FF) | (cpu.instruction.index_addr << 8);
                    // page crossed?
                    cpu.instruction.zp_crossed = opaddr + (cpu.Y as u16) >= 0x100;

                    // if instruction has extra cycle on page crossing and it hasn't happened, we don't get
                    // the extra cycle (finish fetching now)
                    if !cpu.instruction.zp_crossed && extra_cycle
                    {
                        cpu.instruction.cycles_to_fetch = 1;
                    }
                },
                1 => { // if page crossed - add 0x100 to operand address
                    let addr = cpu.instruction.operand_addr;
                    cpu.read_idle(addr);
                    if cpu.instruction.zp_crossed { cpu.instruction.operand_addr += 0x100; }
                },
                _ => panic!("Too many cycles for operand address fetch! ({}) ", cpu.instruction.cycles_to_fetch)
            }
        },
        AddrMode::Indirect => {
            match cpu.instruction.cycles_to_fetch {
                2 => {
                    cpu.instruction.operand_addr = cpu.next_byte() as u16;
                },
                1 => {
                    let addr = cpu.instruction.operand_addr | ((cpu.next_byte() as u16) << 8);
                    cpu.instruction.operand_addr = cpu.read_word_le(addr);
                },
                _ => panic!("Too many cycles for operand address fetch! ({}) ", cpu.instruction.cycles_to_fetch)
            }
        },
        _ => {}
    }

    cpu.instruction.cycles_to_fetch -= 1;
    // fetch complete?
    cpu.instruction.cycles_to_fetch == 0
}

pub fn run(cpu: &mut cpu::CPU) -> bool
{
    match cpu.instruction.opcode
    {
        Op::LDA => {
            if cpu.ba_low { return false; }
            let na = cpu.get_operand();
            cpu.A = na;
            cpu.set_zn_flags(na);
        },
        Op::LDX => {
            if cpu.ba_low { return false; }
            let nx = cpu.get_operand();
            cpu.X = nx;
            cpu.set_zn_flags(nx);
        },
        Op::LDY => {
            if cpu.ba_low { return false; }
            let ny = cpu.get_operand();
            cpu.Y = ny;
            cpu.set_zn_flags(ny);
        },
        Op::STA => {
            let addr = cpu.instruction.operand_addr;
            let val = cpu.A;
            cpu.write_byte(addr, val);
        },
        Op::STX => {
            let addr = cpu.instruction.operand_addr;
            let val = cpu.X;
            cpu.write_byte(addr, val);
        },
        Op::STY => {
            let addr = cpu.instruction.operand_addr;
            let val = cpu.Y;
            cpu.write_byte(addr, val);
        },
        Op::TAX => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.X = cpu.A;
            let x = cpu.X;
            cpu.set_zn_flags(x);
        },
        Op::TAY => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.Y = cpu.A;
            let y = cpu.Y;
            cpu.set_zn_flags(y);
        },
        Op::TXA => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.A = cpu.X;
            let a = cpu.A;
            cpu.set_zn_flags(a);
        },
        Op::TYA => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.A = cpu.Y;
            let a = cpu.A;
            cpu.set_zn_flags(a);
        },
        Op::TSX => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.X = cpu.SP;
            let x = cpu.X;
            cpu.set_zn_flags(x);
        },
        Op::TXS => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.SP = cpu.X;
        },
        Op::PHA => {
            match cpu.instruction.cycles_to_run
            {
                2 => {
                    if cpu.ba_low { return false; }
                    let pc = cpu.PC;
                    cpu.read_idle(pc);
                },
                1 => {
                    let a = cpu.A;
                    cpu.push_byte(a);
                },
                _ => panic!("Wrong number of cycles: {} {}", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::PHP => {
            match cpu.instruction.cycles_to_run
            {
                2 => {
                    if cpu.ba_low { return false; }
                    let pc = cpu.PC;
                    cpu.read_idle(pc);
                },
                1 => {
                    let p = cpu.P;
                    // TODO: break flag?
                    cpu.push_byte(p);
                },
                _ => panic!("Wrong number of cycles: {} {}", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::PLA => {
            if cpu.ba_low { return false; }
            match cpu.instruction.cycles_to_run
            {
                3 => {
                    let pc = cpu.PC;
                    cpu.read_idle(pc);
                },
                2 => {
                    let sp = cpu.SP as u16;
                    cpu.read_idle(sp+1);
                },
                1 => {
                    let a = cpu.pop_byte();
                    cpu.A = a;
                    cpu.set_zn_flags(a);
                },
                _ => panic!("Wrong number of cycles: {} {}", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::PLP => {
            if cpu.ba_low { return false; }
            match cpu.instruction.cycles_to_run
            {
                3 => {
                    let pc = cpu.PC;
                    cpu.read_idle(pc);
                },
                2 => {
                    let sp = cpu.SP as u16;
                    cpu.read_idle(sp+1);
                },
                1 => {
                    let p = cpu.pop_byte();
                    cpu.P = p;
                },
                _ => panic!("Wrong number of cycles: {} {}", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::AND => {
            if cpu.ba_low { return false; }
            let v = cpu.get_operand();
            let na = cpu.A & v;
            cpu.A = na;
            cpu.set_zn_flags(na);
        },
        Op::EOR => {
            if cpu.ba_low { return false; }
            let v = cpu.get_operand();
            let na = cpu.A ^ v;
            cpu.A = na;
            cpu.set_zn_flags(na);
        },
        Op::ORA => {
            if cpu.ba_low { return false; }
            let v = cpu.get_operand();
            let na = cpu.A | v;
            cpu.A = na;
            cpu.set_zn_flags(na);
        },
        Op::BIT => {
            if cpu.ba_low { return false; }
            let v = cpu.get_operand();
            let a = cpu.A;
            cpu.set_status_flag(cpu::StatusFlag::Negative, (v & 0x80) != 0); // TODO: is this ok?
            cpu.set_status_flag(cpu::StatusFlag::Overflow, (v & 0x40) != 0);
            cpu.set_status_flag(cpu::StatusFlag::Zero,     (v & a)    == 0);
        },
        Op::ADC => { // TODO: test decimal mode, check if flag values are correct
            if cpu.ba_low { return false; }
            let v = cpu.get_operand();
            cpu.adc(v);
        },
        Op::SBC => { // TODO: test decimal mode, check if flag values are correct
            if cpu.ba_low { return false; }
            let v = cpu.get_operand();
            cpu.sbc(v);
        },
        Op::CMP => {
            if cpu.ba_low { return false; }
            let v = cpu.get_operand();
            let res = cpu.A as i16 - v as i16;
            cpu.set_status_flag(cpu::StatusFlag::Carry, res >= 0);
            cpu.set_zn_flags(res as u8);
        },
        Op::CPX => {
            if cpu.ba_low { return false; }
            let v = cpu.get_operand();
            let res = cpu.X as i16 - v as i16;
            cpu.set_status_flag(cpu::StatusFlag::Carry, res >= 0);
            cpu.set_zn_flags(res as u8);
        },
        Op::CPY => {
            if cpu.ba_low { return false; }
            let v = cpu.get_operand();
            let res = cpu.Y as i16 - v as i16;
            cpu.set_status_flag(cpu::StatusFlag::Carry, res >= 0);
            cpu.set_zn_flags(res as u8);
        },
        Op::INC => {
            let v = (Wrapping(cpu.instruction.rmw_buffer) + Wrapping(0x01)).0;
            let addr = cpu.instruction.operand_addr;
            cpu.write_byte(addr, v);
            cpu.set_zn_flags(v);
        },
        Op::INX => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.X = (Wrapping(cpu.X) + Wrapping(0x01)).0;
            let x = cpu.X;
            cpu.set_zn_flags(x);
        },
        Op::INY => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.Y = (Wrapping(cpu.Y) + Wrapping(0x01)).0;
            let y = cpu.Y;
            cpu.set_zn_flags(y);
        },
        Op::DEC => {
            let v = (Wrapping(cpu.instruction.rmw_buffer) - Wrapping(0x01)).0;
            let addr = cpu.instruction.operand_addr;
            cpu.write_byte(addr, v);
            cpu.set_zn_flags(v);
        },
        Op::DEX => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.X = (Wrapping(cpu.X) - Wrapping(0x01)).0;
            let x = cpu.X;
            cpu.set_zn_flags(x);
        },
        Op::DEY => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.Y = (Wrapping(cpu.Y) - Wrapping(0x01)).0;
            let y = cpu.Y;
            cpu.set_zn_flags(y);
        },
        Op::ASL => {
            if cpu.ba_low {
                match cpu.instruction.addr_mode {
                    AddrMode::Accumulator => {return false; },
                    _ => (),
                }
            }
            let v = cpu.get_operand();
            cpu.set_status_flag(cpu::StatusFlag::Carry, (v & 0x80) != 0);
            let res = v << 1;
            cpu.set_operand(res);
            cpu.set_zn_flags(res);
        },
        Op::LSR => {
            if cpu.ba_low {
                match cpu.instruction.addr_mode {
                    AddrMode::Accumulator => {return false; },
                    _ => (),
                }
            }
            let v = cpu.get_operand();
            cpu.set_status_flag(cpu::StatusFlag::Carry, (v & 0x01) != 0);
            let res = v >> 1;
            cpu.set_operand(res);
            cpu.set_zn_flags(res);
        },
        Op::ROL => {
            if cpu.ba_low {
                match cpu.instruction.addr_mode {
                    AddrMode::Accumulator => {return false; },
                    _ => (),
                }
            }
            let c = cpu.get_status_flag(cpu::StatusFlag::Carry);
            let v = cpu.get_operand();
            cpu.set_status_flag(cpu::StatusFlag::Carry, (v & 0x80) != 0);
            let mut res = v << 1;
            if c
            {
                res |= 0x01;
            }
            cpu.set_operand(res);
            cpu.set_zn_flags(res);
        },
        Op::ROR => {
            if cpu.ba_low {
                match cpu.instruction.addr_mode {
                    AddrMode::Accumulator => {return false; },
                    _ => (),
                }
            }
            let c = cpu.get_status_flag(cpu::StatusFlag::Carry);
            let v = cpu.get_operand();
            cpu.set_status_flag(cpu::StatusFlag::Carry, (v & 0x01) != 0);
            let mut res = v >> 1;
            if c
            {
                res |= 0x80;
            }
            cpu.set_operand(res);
            cpu.set_zn_flags(res);
        },
        Op::JMP => { // TODO: is this ok?
            if cpu.ba_low { return false; }
            match cpu.instruction.cycles_to_run
            {
                2 => {
                    //cpu.instruction.cycles_to_run -= 1;
                },
                1 | 0 => {
                    cpu.PC = cpu.instruction.operand_addr;
                    cpu.instruction.cycles_to_run = 1;
                },
                _ => panic!("Wrong number of cycles: {} {} ", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::JSR => { // TODO: is this ok?
            match cpu.instruction.cycles_to_run
            {
                3 => {
                    // TODO: break down PC push to 2 byte instructions?
                },
                2 => {
                    let pc = cpu.PC - 0x0001;
                    cpu.push_word(pc);
                },
                1  => {
                    if cpu.ba_low { return false; }
                    let pc = cpu.PC;
                    cpu.read_idle(pc);
                    cpu.PC = cpu.instruction.operand_addr;
                },
                _ => panic!("Wrong number of cycles: {} {} ", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::RTS => {
            if cpu.ba_low { return false; }

            match cpu.instruction.cycles_to_run
            {
                5 => {
                    let pc = cpu.PC;
                    cpu.read_idle(pc);
                },
                4 => {
                    let sp = cpu.SP as u16;
                    cpu.read_idle(sp + 1);
                },
                3 => {
                    let pc_lo = cpu.pop_byte() as u16;
                    cpu.PC = pc_lo;
                },
                2 => {
                    let pc_hi = cpu.pop_byte() as u16;
                    cpu.PC |= pc_hi << 8;
                },
                1  => {
                    let pc = cpu.PC;
                    cpu.read_idle(pc+1);
                    cpu.PC += 1;
                },
                _ => panic!("Wrong number of cycles: {} {} ", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        // branching ops: (TODO: take into account forward/back branching?)
        // take 2 cycles (fetch + execute) if no branch is taken
        // 3 cycles if branch is taken, no page crossed
        // 4 cycles if branch is taken, page crossed
        Op::BCC => {
            match cpu.instruction.cycles_to_run
            {
                3 => {
                    if cpu.ba_low { return false; }
                    if !cpu.get_status_flag(cpu::StatusFlag::Carry)
                    {
                        let addr = cpu.instruction.operand_addr;
                        let pc = cpu.PC;
                        cpu.instruction.zp_crossed = (addr >> 8) != (pc >> 8);
                    }
                    else
                    {
                        // no branching - finish instruction after only 2 cycles
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                2 => {
                    if !cpu.instruction.zp_crossed
                    {
                        cpu.first_irq_cycle += 1;
                        cpu.first_nmi_cycle += 1;
                    }
                    if cpu.ba_low { return false; }
                    
                    let pc = cpu.PC;
                    let addr = cpu.instruction.operand_addr;
                    cpu.read_idle(pc);
                    cpu.PC = addr;

                    if !cpu.instruction.zp_crossed
                    {
                        // no page crossing - finish instruction after only 3 cycle
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                1 => {
                    if cpu.ba_low { return false; }
                    let pc = cpu.PC;
                    cpu.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                },
                _ => panic!("Wrong number of cycles: {} {} ", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::BCS => {
            match cpu.instruction.cycles_to_run
            {
                3 => {
                    if cpu.ba_low { return false; }
                    if cpu.get_status_flag(cpu::StatusFlag::Carry)
                    {
                        let addr = cpu.instruction.operand_addr;
                        let pc = cpu.PC;
                        cpu.instruction.zp_crossed = (addr >> 8) != (pc >> 8);
                    }
                    else
                    {
                        // no branching - finish instruction after only 2 cycles
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                2 => {
                    if !cpu.instruction.zp_crossed
                    {
                        cpu.first_irq_cycle += 1;
                        cpu.first_nmi_cycle += 1;
                    }
                    if cpu.ba_low { return false; }
                    
                    let pc = cpu.PC;
                    let addr = cpu.instruction.operand_addr;
                    cpu.read_idle(pc);
                    cpu.PC = addr;

                    if !cpu.instruction.zp_crossed
                    {
                        // no page crossing - finish instruction after only 3 cycle
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                1 => {
                    if cpu.ba_low { return false; }
                    let pc = cpu.PC;
                    cpu.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                },
                _ => panic!("Wrong number of cycles: {} {} ", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::BEQ => {
            match cpu.instruction.cycles_to_run
            {
                3 => {
                    if cpu.ba_low { return false; }
                    if cpu.get_status_flag(cpu::StatusFlag::Zero)
                    {
                        let addr = cpu.instruction.operand_addr;
                        let pc = cpu.PC;
                        cpu.instruction.zp_crossed = (addr >> 8) != (pc >> 8);
                    }
                    else
                    {
                        // no branching - finish instruction after only 2 cycles
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                2 => {
                    if !cpu.instruction.zp_crossed
                    {
                        cpu.first_irq_cycle += 1;
                        cpu.first_nmi_cycle += 1;
                    }
                    if cpu.ba_low { return false; }
                    
                    let pc = cpu.PC;
                    let addr = cpu.instruction.operand_addr;
                    cpu.read_idle(pc);
                    cpu.PC = addr;

                    if !cpu.instruction.zp_crossed
                    {
                        // no page crossing - finish instruction after only 3 cycle
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                1 => {
                    if cpu.ba_low { return false; }
                    let pc = cpu.PC;
                    cpu.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                },
                _ => panic!("Wrong number of cycles: {} {} ", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::BNE => {
            match cpu.instruction.cycles_to_run
            {
                3 => {
                    if cpu.ba_low { return false; }
                    if !cpu.get_status_flag(cpu::StatusFlag::Zero)
                    {
                        let addr = cpu.instruction.operand_addr;
                        let pc = cpu.PC;
                        cpu.instruction.zp_crossed = (addr >> 8) != (pc >> 8);
                    }
                    else
                    {
                        // no branching - finish instruction after only 2 cycles
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                2 => {
                    if !cpu.instruction.zp_crossed
                    {
                        cpu.first_irq_cycle += 1;
                        cpu.first_nmi_cycle += 1;
                    }
                    if cpu.ba_low { return false; }
                    
                    let pc = cpu.PC;
                    let addr = cpu.instruction.operand_addr;
                    cpu.read_idle(pc);
                    cpu.PC = addr;
                    if !cpu.instruction.zp_crossed
                    {
                        // no page crossing - finish instruction after only 3 cycle
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                1 => {
                    if cpu.ba_low { return false; }
                    let pc = cpu.PC;
                    cpu.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                },
                _ => panic!("Wrong number of cycles: {} {} ", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::BMI => {
            match cpu.instruction.cycles_to_run
            {
                3 => {
                    if cpu.ba_low { return false; }
                    if cpu.get_status_flag(cpu::StatusFlag::Negative)
                    {
                        let addr = cpu.instruction.operand_addr;
                        let pc = cpu.PC;
                        cpu.instruction.zp_crossed = (addr >> 8) != (pc >> 8);
                    }
                    else
                    {
                        // no branching - finish instruction after only 2 cycles
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                2 => {
                    if !cpu.instruction.zp_crossed
                    {
                        cpu.first_irq_cycle += 1;
                        cpu.first_nmi_cycle += 1;
                    }
                    if cpu.ba_low { return false; }
                    
                    let pc = cpu.PC;
                    let addr = cpu.instruction.operand_addr;
                    cpu.read_idle(pc);
                    cpu.PC = addr;

                    if !cpu.instruction.zp_crossed
                    {
                        // no page crossing - finish instruction after only 3 cycle
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                1 => {
                    if cpu.ba_low { return false; }
                    let pc = cpu.PC;
                    cpu.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                },
                _ => panic!("Wrong number of cycles: {} {} ", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::BPL => {
            match cpu.instruction.cycles_to_run
            {
                3 => {
                    if cpu.ba_low { return false; }
                    if !cpu.get_status_flag(cpu::StatusFlag::Negative)
                    {
                        let addr = cpu.instruction.operand_addr;
                        let pc = cpu.PC;
                        cpu.instruction.zp_crossed = (addr >> 8) != (pc >> 8);
                    }
                    else
                    {
                        // no branching - finish instruction after only 2 cycles
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                2 => {
                    if !cpu.instruction.zp_crossed
                    {
                        cpu.first_irq_cycle += 1;
                        cpu.first_nmi_cycle += 1;
                    }
                    if cpu.ba_low { return false; }
                    
                    let pc = cpu.PC;
                    let addr = cpu.instruction.operand_addr;
                    cpu.read_idle(pc);
                    cpu.PC = addr;

                    if !cpu.instruction.zp_crossed
                    {
                        // no page crossing - finish instruction after only 3 cycle
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                1 => {
                    if cpu.ba_low { return false; }
                    let pc = cpu.PC;
                    cpu.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                },
                _ => panic!("Wrong number of cycles: {} {} ", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::BVC => {
            match cpu.instruction.cycles_to_run
            {
                3 => {
                    if cpu.ba_low { return false; }
                    if !cpu.get_status_flag(cpu::StatusFlag::Overflow)
                    {
                        let addr = cpu.instruction.operand_addr;
                        let pc = cpu.PC;
                        cpu.instruction.zp_crossed = (addr >> 8) != (pc >> 8);
                    }
                    else
                    {
                        // no branching - finish instruction after only 2 cycles
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                2 => {
                    if !cpu.instruction.zp_crossed
                    {
                        cpu.first_irq_cycle += 1;
                        cpu.first_nmi_cycle += 1;
                    }
                    if cpu.ba_low { return false; }
                    
                    let pc = cpu.PC;
                    let addr = cpu.instruction.operand_addr;
                    cpu.read_idle(pc);
                    cpu.PC = addr;

                    if !cpu.instruction.zp_crossed
                    {
                        // no page crossing - finish instruction after only 3 cycle
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                1 => {
                    if cpu.ba_low { return false; }
                    let pc = cpu.PC;
                    cpu.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                },
                _ => panic!("Wrong number of cycles: {} {} ", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::BVS => {
            match cpu.instruction.cycles_to_run
            {
                3 => {
                    if cpu.ba_low { return false; }
                    if cpu.get_status_flag(cpu::StatusFlag::Overflow)
                    {
                        let addr = cpu.instruction.operand_addr;
                        let pc = cpu.PC;
                        cpu.instruction.zp_crossed = (addr >> 8) != (pc >> 8);
                    }
                    else
                    {
                        // no branching - finish instruction after only 2 cycles
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                2 => {
                    if !cpu.instruction.zp_crossed
                    {
                        cpu.first_irq_cycle += 1;
                        cpu.first_nmi_cycle += 1;
                    }
                    if cpu.ba_low { return false; }
                    
                    let pc = cpu.PC;
                    let addr = cpu.instruction.operand_addr;
                    cpu.read_idle(pc);
                    cpu.PC = addr;

                    if !cpu.instruction.zp_crossed
                    {
                        // no page crossing - finish instruction after only 3 cycle
                        cpu.instruction.cycles_to_run = 1;
                    }
                },
                1 => {
                    if cpu.ba_low { return false; }
                    let pc = cpu.PC;
                    cpu.read_idle(pc); // TODO: not sure if we shouldn't read different val here depending on branching fw/bckw
                },
                _ => panic!("Wrong number of cycles: {} {} ", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::CLC => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.set_status_flag(cpu::StatusFlag::Carry, false);
        },
        Op::CLD => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.set_status_flag(cpu::StatusFlag::DecimalMode, false);
        },
        Op::CLI => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.set_status_flag(cpu::StatusFlag::InterruptDisable, false);
        },
        Op::CLV => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.set_status_flag(cpu::StatusFlag::Overflow, false);
        },
        Op::SEC => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.set_status_flag(cpu::StatusFlag::Carry, true);
        },
        Op::SED => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.set_status_flag(cpu::StatusFlag::DecimalMode, true);
        },
        Op::SEI => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
            cpu.set_status_flag(cpu::StatusFlag::InterruptDisable, true);
        },
        Op::BRK => { // TODO: is this ok? do we have to break down new PC value to 2 cycles? read_word ok here?
            match cpu.instruction.cycles_to_run
            {
                6 => {
                    if cpu.ba_low { return false; }
                    let pc = cpu.PC + 0x0001;
                    cpu.read_idle(pc);
                },
                5 => {
                    let pc = cpu.PC + 0x0001;
                    cpu.push_byte(((pc >> 8) & 0xFF) as u8);
                },
                4 => {
                    let pc = cpu.PC + 0x0001;
                    cpu.push_byte((pc & 0xFF) as u8);
                },
                3 => {
                    cpu.set_status_flag(cpu::StatusFlag::Break, true);
                    let p = cpu.P;
                    cpu.push_byte(p);
                    cpu.set_status_flag(cpu::StatusFlag::InterruptDisable, true);
                    if cpu.nmi
                    {
                        cpu.nmi_cycles_left = 7;
                        cpu.state = cpu::CPUState::ProcessNMI;
                    }
                },
                2 => {
                    cpu.first_nmi_cycle += 1; // delay NMI
                },
                1  => {
                    //println!("Received BRK instruction at ${:04X}", cpu.PC-1);
                    cpu.PC = cpu.read_word_le(cpu::IRQ_VECTOR);
                },
                _ => panic!("Wrong number of cycles: {} {} ", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        Op::NOP => {
            if cpu.ba_low { return false; }
            let pc = cpu.PC;
            cpu.read_idle(pc);
        },
        Op::RTI => { // TODO is this ok?
            if cpu.ba_low { return false; }

            match cpu.instruction.cycles_to_run
            {
                5 => {
                    let pc = cpu.PC;
                    cpu.read_idle(pc);
                },
                4 => {
                    let sp = cpu.SP as u16;
                    cpu.read_idle(sp + 1);
                },
                3 => {
                    let p = cpu.pop_byte();
                    cpu.P = p;
                },
                2 => {
                    let pc_lo = cpu.pop_byte() as u16;
                    cpu.PC = pc_lo;
                },
                1  => {
                    let pc_hi = cpu.pop_byte() as u16;
                    cpu.PC |= pc_hi << 8;
                },
                _ => panic!("Wrong number of cycles: {} {} ", cpu.instruction, cpu.instruction.cycles_to_run)
            }
        },
        // forbidden ops
        Op::HLT => {
            panic!("Received HLT instruction at ${:04X}", cpu.PC-1);
        },
        Op::SLO => {
            let mut v = cpu.instruction.rmw_buffer;
            let nc = (v & 0x80) != 0;
            cpu.set_status_flag(cpu::StatusFlag::Carry, nc);
            v <<= 1;
            cpu.set_operand(v);
            let na = cpu.A | v;
            cpu.A = na;
            cpu.set_zn_flags(na);
        },
        Op::ANC => {
            let v = cpu.get_operand();
            let na = cpu.A & v;
            cpu.set_zn_flags(na);
            let n = cpu.get_status_flag(cpu::StatusFlag::Negative);
            cpu.set_status_flag(cpu::StatusFlag::Carry, n);
        },
        Op::RLA => {
            let tmp = cpu.instruction.rmw_buffer & 0x80;
            let c = cpu.get_status_flag(cpu::StatusFlag::Carry);
            let mut v = cpu.instruction.rmw_buffer << 1;
            if c
            {
                v |= 1;
            }

            cpu.set_status_flag(cpu::StatusFlag::Carry, tmp != 0);
            cpu.set_operand(v);
            let na = cpu.A & v;
            cpu.A = na;
            cpu.set_zn_flags(na);
        },
        Op::SRE => {
            let mut v = cpu.instruction.rmw_buffer;
            let nc = (v & 0x01) != 0;
            cpu.set_status_flag(cpu::StatusFlag::Carry, nc);
            v >>= 1;
            cpu.set_operand(v);
            let na = cpu.A ^ v;
            cpu.A = na;
            cpu.set_zn_flags(na);
        },
        Op::RRA => {
            let mut v = cpu.instruction.rmw_buffer;
            let tmp = v & 0x01;
            let c = cpu.get_status_flag(cpu::StatusFlag::Carry);
            v >>= 1;
            if c
            {
                v |= 0x80;
            }
            cpu.set_status_flag(cpu::StatusFlag::Carry, tmp != 0);
            cpu.adc(v);
        },
        Op::SAX => {
            let v = cpu.A & cpu.X;
            cpu.set_operand(v);
        },
        Op::AHX => {
            let addr = cpu.instruction.operand_addr;
            let addr_hi = cpu.instruction.index_addr as u8;
            let y = cpu.Y;
            cpu.write_byte(addr, y & (addr_hi + 1));
        },
        Op::TAS => {
            let addr = cpu.instruction.operand_addr;
            let addr_hi = cpu.instruction.index_addr as u8;
            let a = cpu.A;
            let x = cpu.X;
            cpu.SP = a & x;
            cpu.write_byte(addr, (a & x) & (addr_hi + 1));
        },
        Op::SHY => {
            let addr = cpu.instruction.operand_addr;
            let addr_hi = cpu.instruction.index_addr as u8;
            let a = cpu.A;
            let x = cpu.X;
            cpu.write_byte(addr, a & x & (addr_hi + 1));
        },
        Op::SHX => {
            let addr = cpu.instruction.operand_addr;
            let addr_hi = cpu.instruction.index_addr as u8;
            let x = cpu.X;
            cpu.write_byte(addr, x & (addr_hi + 1));
        },
        Op::LAX => {
            if cpu.ba_low { return false; }
            let nv = cpu.get_operand();
            cpu.A = nv;
            cpu.X = nv;
            cpu.set_zn_flags(nv);
        }, 
        Op::DCP => {
            let v = (Wrapping(cpu.instruction.rmw_buffer) - Wrapping(0x01)).0;
            cpu.set_operand(v);
            let diff = (Wrapping(cpu.A as u16) - Wrapping(v as u16)).0;
            cpu.set_zn_flags(diff as u8);
            cpu.set_status_flag(cpu::StatusFlag::Carry, (diff & 0x0100) == 0);
        },
        Op::ISC => {
            cpu.instruction.rmw_buffer += 1;
            let v = cpu.instruction.rmw_buffer;
            cpu.set_operand(v);
            cpu.sbc(v);
        },
        _ => panic!("Unknown instruction: {} at ${:04X}", cpu.instruction, cpu.PC)
    }


    cpu.instruction.cycles_to_run -= 1;
    // instruction finished execution?
    cpu.instruction.cycles_to_run == 0
}

// num cycles represents the *max* number of cycles that the instruction can take to execute (so taking into account extra cycles for branching, page crosses etc.)
pub fn get_instruction(opcode: u8) -> Option<(Op, u8, bool, AddrMode)>
{
    Some(match opcode
         {
             /* ** documented instructions ** */
             /* BRK     */ 0x00 => (Op::BRK, 7, false, AddrMode::Implied),
             /* ORA_izx */ 0x01 => (Op::ORA, 6, false, AddrMode::IndexedIndirectX),
             /* ORA_zp  */ 0x05 => (Op::ORA, 3, false, AddrMode::Zeropage),
             /* ASL_zp  */ 0x06 => (Op::ASL, 5,  true, AddrMode::Zeropage), 
             /* PHP     */ 0x08 => (Op::PHP, 3, false, AddrMode::Implied),
             /* ORA_imm */ 0x09 => (Op::ORA, 2, false, AddrMode::Immediate),
             /* ASL     */ 0x0A => (Op::ASL, 2, false, AddrMode::Accumulator),
             /* ORA_abs */ 0x0D => (Op::ORA, 4, false, AddrMode::Absolute),
             /* ASL_abs */ 0x0E => (Op::ASL, 6,  true, AddrMode::Absolute),
             /* BPL_rel */ 0x10 => (Op::BPL, 4, false, AddrMode::Relative), // add 1 cycle if page boundary is crossed
             /* ORA_izy */ 0x11 => (Op::ORA, 6, false, AddrMode::IndirectIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* ORA_zpx */ 0x15 => (Op::ORA, 4, false, AddrMode::ZeropageIndexedX),
             /* ASL_zpx */ 0x16 => (Op::ASL, 6,  true, AddrMode::ZeropageIndexedX),
             /* CLC     */ 0x18 => (Op::CLC, 2, false, AddrMode::Implied),
             /* ORA_aby */ 0x19 => (Op::ORA, 5, false, AddrMode::AbsoluteIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* ORA_abx */ 0x1D => (Op::ORA, 5, false, AddrMode::AbsoluteIndexedX(true)), // add 1 cycle if page boundary is crossed
             /* ASL_abx */ 0x1E => (Op::ASL, 7,  true, AddrMode::AbsoluteIndexedX(false)),
             /* JSR_abs */ 0x20 => (Op::JSR, 6, false, AddrMode::Absolute),
             /* AND_izx */ 0x21 => (Op::AND, 6, false, AddrMode::IndexedIndirectX),
             /* BIT_zp  */ 0x24 => (Op::BIT, 3, false, AddrMode::Zeropage),
             /* AND_zp  */ 0x25 => (Op::AND, 3, false, AddrMode::Zeropage),
             /* ROL_zp  */ 0x26 => (Op::ROL, 5,  true, AddrMode::Zeropage),
             /* PLP     */ 0x28 => (Op::PLP, 4, false, AddrMode::Implied),
             /* AND_imm */ 0x29 => (Op::AND, 2, false, AddrMode::Immediate),
             /* ROL     */ 0x2A => (Op::ROL, 2, false, AddrMode::Accumulator),
             /* BIT_abs */ 0x2C => (Op::BIT, 4, false, AddrMode::Absolute),
             /* AND_abs */ 0x2D => (Op::AND, 4, false, AddrMode::Absolute),
             /* ROL_abs */ 0x2E => (Op::ROL, 6,  true, AddrMode::Absolute),
             /* BMI_rel */ 0x30 => (Op::BMI, 4, false, AddrMode::Relative), // add 1 cycle if page boundary is crossed
             /* AND_izy */ 0x31 => (Op::AND, 6, false, AddrMode::IndirectIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* AND_zpx */ 0x35 => (Op::AND, 4, false, AddrMode::ZeropageIndexedX),
             /* ROL_zpx */ 0x36 => (Op::ROL, 6,  true, AddrMode::ZeropageIndexedX),
             /* SEC     */ 0x38 => (Op::SEC, 2, false, AddrMode::Implied),
             /* AND_aby */ 0x39 => (Op::AND, 5, false, AddrMode::AbsoluteIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* AND_abx */ 0x3D => (Op::AND, 5, false, AddrMode::AbsoluteIndexedX(true)), // add 1 cycle if page boundary is crossed
             /* ROL_abx */ 0x3E => (Op::ROL, 7,  true, AddrMode::AbsoluteIndexedX(false)),
             /* RTI     */ 0x40 => (Op::RTI, 6, false, AddrMode::Implied),
             /* EOR_izx */ 0x41 => (Op::EOR, 6, false, AddrMode::IndexedIndirectX),
             /* EOR_zp  */ 0x45 => (Op::EOR, 3, false, AddrMode::Zeropage),
             /* LSR_zp  */ 0x46 => (Op::LSR, 5,  true, AddrMode::Zeropage),
             /* PHA     */ 0x48 => (Op::PHA, 3, false, AddrMode::Implied),
             /* EOR_imm */ 0x49 => (Op::EOR, 2, false, AddrMode::Immediate),
             /* LSR     */ 0x4A => (Op::LSR, 2, false, AddrMode::Accumulator),
             /* JMP_abs */ 0x4C => (Op::JMP, 3, false, AddrMode::Absolute),
             /* EOR_abs */ 0x4D => (Op::EOR, 4, false, AddrMode::Absolute),
             /* LSR_abs */ 0x4E => (Op::LSR, 6,  true, AddrMode::Absolute),
             /* BVC_rel */ 0x50 => (Op::BVC, 4, false, AddrMode::Relative), // add 1 cycle if page boundary is crossed
             /* EOR_izy */ 0x51 => (Op::EOR, 6, false, AddrMode::IndirectIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* EOR_zpx */ 0x55 => (Op::EOR, 4, false, AddrMode::ZeropageIndexedX),
             /* LSR_zpx */ 0x56 => (Op::LSR, 6,  true, AddrMode::ZeropageIndexedX),
             /* CLI     */ 0x58 => (Op::CLI, 2, false, AddrMode::Implied),
             /* EOR_aby */ 0x59 => (Op::EOR, 5, false, AddrMode::AbsoluteIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* EOR_abx */ 0x5D => (Op::EOR, 5, false, AddrMode::AbsoluteIndexedX(true)), // add 1 cycle if page boundary is crossed
             /* LSR_abx */ 0x5E => (Op::LSR, 7,  true, AddrMode::AbsoluteIndexedX(false)),
             /* RTS     */ 0x60 => (Op::RTS, 6, false, AddrMode::Implied),
             /* ADC_izx */ 0x61 => (Op::ADC, 6, false, AddrMode::IndexedIndirectX),
             /* ADC_zp  */ 0x65 => (Op::ADC, 3, false, AddrMode::Zeropage),
             /* ROR_zp  */ 0x66 => (Op::ROR, 5,  true, AddrMode::Zeropage),
             /* PLA     */ 0x68 => (Op::PLA, 4, false, AddrMode::Implied),
             /* ADC_imm */ 0x69 => (Op::ADC, 2, false, AddrMode::Immediate),
             /* ROR     */ 0x6A => (Op::ROR, 2, false, AddrMode::Accumulator),
             /* JMP_ind */ 0x6C => (Op::JMP, 5, false, AddrMode::Indirect),
             /* ADC_abs */ 0x6D => (Op::ADC, 4, false, AddrMode::Absolute),
             /* ROR_abs */ 0x6E => (Op::ROR, 6,  true, AddrMode::Absolute),
             /* BVS_rel */ 0x70 => (Op::BVS, 4, false, AddrMode::Relative), // add 1 cycle if page boundary is crossed
             /* ADC_izy */ 0x71 => (Op::ADC, 6, false, AddrMode::IndirectIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* ADC_zpx */ 0x75 => (Op::ADC, 4, false, AddrMode::ZeropageIndexedX),
             /* ROR_zpx */ 0x76 => (Op::ROR, 6,  true, AddrMode::ZeropageIndexedX),
             /* SEI     */ 0x78 => (Op::SEI, 2, false, AddrMode::Implied),
             /* ADC_aby */ 0x79 => (Op::ADC, 5, false, AddrMode::AbsoluteIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* ADC_abx */ 0x7D => (Op::ADC, 5, false, AddrMode::AbsoluteIndexedX(true)), // add 1 cycle if page boundary is crossed
             /* ROR_abx */ 0x7E => (Op::ROR, 7,  true, AddrMode::AbsoluteIndexedX(false)),
             /* STA_izx */ 0x81 => (Op::STA, 6, false, AddrMode::IndexedIndirectX),
             /* STY_zp  */ 0x84 => (Op::STY, 3, false, AddrMode::Zeropage),
             /* STA_zp  */ 0x85 => (Op::STA, 3, false, AddrMode::Zeropage),
             /* STX_zp  */ 0x86 => (Op::STX, 3, false, AddrMode::Zeropage),
             /* DEY     */ 0x88 => (Op::DEY, 2, false, AddrMode::Implied),
             /* TXA     */ 0x8A => (Op::TXA, 2, false, AddrMode::Implied),
             /* STY_abs */ 0x8C => (Op::STY, 4, false, AddrMode::Absolute),
             /* STA_abs */ 0x8D => (Op::STA, 4, false, AddrMode::Absolute),
             /* STX_abs */ 0x8E => (Op::STX, 4, false, AddrMode::Absolute),
             /* BCC_rel */ 0x90 => (Op::BCC, 4, false, AddrMode::Relative), // add 1 cycle if page boundary is crossed
             /* STA_izy */ 0x91 => (Op::STA, 6, false, AddrMode::IndirectIndexedY(false)),
             /* STY_zpx */ 0x94 => (Op::STY, 4, false, AddrMode::ZeropageIndexedX),
             /* STA_zpx */ 0x95 => (Op::STA, 4, false, AddrMode::ZeropageIndexedX),
             /* STX_zpy */ 0x96 => (Op::STX, 4, false, AddrMode::ZeropageIndexedY),
             /* TYA     */ 0x98 => (Op::TYA, 2, false, AddrMode::Implied),
             /* STA_aby */ 0x99 => (Op::STA, 5, false, AddrMode::AbsoluteIndexedY(false)),
             /* TXS     */ 0x9A => (Op::TXS, 2, false, AddrMode::Implied),
             /* STA_abx */ 0x9D => (Op::STA, 5, false, AddrMode::AbsoluteIndexedX(false)),
             /* LDY_imm */ 0xA0 => (Op::LDY, 2, false, AddrMode::Immediate),
             /* LDA_izx */ 0xA1 => (Op::LDA, 6, false, AddrMode::IndexedIndirectX),
             /* LDX_imm */ 0xA2 => (Op::LDX, 2, false, AddrMode::Immediate),
             /* LDY_zp  */ 0xA4 => (Op::LDY, 3, false, AddrMode::Zeropage),
             /* LDA_zp  */ 0xA5 => (Op::LDA, 3, false, AddrMode::Zeropage),
             /* LDX_zp  */ 0xA6 => (Op::LDX, 3, false, AddrMode::Zeropage),
             /* TAY     */ 0xA8 => (Op::TAY, 2, false, AddrMode::Implied),
             /* LDA_imm */ 0xA9 => (Op::LDA, 2, false, AddrMode::Immediate),
             /* TAX     */ 0xAA => (Op::TAX, 2, false, AddrMode::Implied),
             /* LDY_abs */ 0xAC => (Op::LDY, 4, false, AddrMode::Absolute),
             /* LDA_abs */ 0xAD => (Op::LDA, 4, false, AddrMode::Absolute),
             /* LDX_abs */ 0xAE => (Op::LDX, 4, false, AddrMode::Absolute),
             /* BCS_rel */ 0xB0 => (Op::BCS, 4, false, AddrMode::Relative), // add 1 cycle if page boundary is crossed
             /* LDA_izy */ 0xB1 => (Op::LDA, 6, false, AddrMode::IndirectIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* LDY_zpx */ 0xB4 => (Op::LDY, 4, false, AddrMode::ZeropageIndexedX),
             /* LDA_zpx */ 0xB5 => (Op::LDA, 4, false, AddrMode::ZeropageIndexedX),
             /* LDX_zpy */ 0xB6 => (Op::LDX, 4, false, AddrMode::ZeropageIndexedY),
             /* CLV     */ 0xB8 => (Op::CLV, 2, false, AddrMode::Implied),
             /* LDA_aby */ 0xB9 => (Op::LDA, 5, false, AddrMode::AbsoluteIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* TSX     */ 0xBA => (Op::TSX, 2, false, AddrMode::Implied),
             /* LDY_abx */ 0xBC => (Op::LDY, 5, false, AddrMode::AbsoluteIndexedX(true)), // add 1 cycle if page boundary is crossed
             /* LDA_abx */ 0xBD => (Op::LDA, 5, false, AddrMode::AbsoluteIndexedX(true)), // add 1 cycle if page boundary is crossed
             /* LDX_aby */ 0xBE => (Op::LDX, 5, false, AddrMode::AbsoluteIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* CPY_imm */ 0xC0 => (Op::CPY, 2, false, AddrMode::Immediate),
             /* CMP_izx */ 0xC1 => (Op::CMP, 6, false, AddrMode::IndexedIndirectX),
             /* CPY_zp  */ 0xC4 => (Op::CPY, 3, false, AddrMode::Zeropage),
             /* CMP_zp  */ 0xC5 => (Op::CMP, 3, false, AddrMode::Zeropage),
             /* DEC_zp  */ 0xC6 => (Op::DEC, 5,  true, AddrMode::Zeropage),
             /* INY     */ 0xC8 => (Op::INY, 2, false, AddrMode::Implied),
             /* CMP_imm */ 0xC9 => (Op::CMP, 2, false, AddrMode::Immediate),
             /* DEX     */ 0xCA => (Op::DEX, 2, false, AddrMode::Implied),
             /* CPY_abs */ 0xCC => (Op::CPY, 4, false, AddrMode::Absolute),
             /* CMP_abs */ 0xCD => (Op::CMP, 4, false, AddrMode::Absolute),
             /* DEC_abs */ 0xCE => (Op::DEC, 6,  true, AddrMode::Absolute),
             /* BNE_rel */ 0xD0 => (Op::BNE, 4, false, AddrMode::Relative), // add 1 cycle if page boundary is crossed
             /* CMP_izy */ 0xD1 => (Op::CMP, 6, false, AddrMode::IndirectIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* CMP_zpx */ 0xD5 => (Op::CMP, 4, false, AddrMode::ZeropageIndexedX),
             /* DEC_zpx */ 0xD6 => (Op::DEC, 6,  true, AddrMode::ZeropageIndexedX),
             /* CLD     */ 0xD8 => (Op::CLD, 2, false, AddrMode::Implied),
             /* CMP_aby */ 0xD9 => (Op::CMP, 5, false, AddrMode::AbsoluteIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* CMP_abx */ 0xDD => (Op::CMP, 5, false, AddrMode::AbsoluteIndexedX(true)), // add 1 cycle if page boundary is crossed
             /* DEC_abx */ 0xDE => (Op::DEC, 7,  true, AddrMode::AbsoluteIndexedX(false)),
             /* CPX_imm */ 0xE0 => (Op::CPX, 2, false, AddrMode::Immediate),
             /* SBC_izx */ 0xE1 => (Op::SBC, 6, false, AddrMode::IndexedIndirectX),
             /* CPX_zp  */ 0xE4 => (Op::CPX, 3, false, AddrMode::Zeropage),
             /* SBC_zp  */ 0xE5 => (Op::SBC, 3, false, AddrMode::Zeropage),
             /* INC_zp  */ 0xE6 => (Op::INC, 5,  true, AddrMode::Zeropage),
             /* INX     */ 0xE8 => (Op::INX, 2, false, AddrMode::Implied),
             /* SBC_imm */ 0xE9 => (Op::SBC, 2, false, AddrMode::Immediate),
             /* NOP     */ 0xEA => (Op::NOP, 2, false, AddrMode::Implied),
             /* CPX     */ 0xEC => (Op::CPX, 4, false, AddrMode::Absolute),
             /* SBC_abs */ 0xED => (Op::SBC, 4, false, AddrMode::Absolute),
             /* INC_abs */ 0xEE => (Op::INC, 6,  true, AddrMode::Absolute),
             /* BEQ_rel */ 0xF0 => (Op::BEQ, 4, false, AddrMode::Relative), // add 1 cycle if page boundary is crossed
             /* SBC_izy */ 0xF1 => (Op::SBC, 6, false, AddrMode::IndirectIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* SBC_zpx */ 0xF5 => (Op::SBC, 4, false, AddrMode::ZeropageIndexedX),
             /* INC_zpx */ 0xF6 => (Op::INC, 6,  true, AddrMode::ZeropageIndexedX),
             /* SED     */ 0xF8 => (Op::SED, 2, false, AddrMode::Implied),
             /* SBC_aby */ 0xF9 => (Op::SBC, 5, false, AddrMode::AbsoluteIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* SBC_abx */ 0xFD => (Op::SBC, 5, false, AddrMode::AbsoluteIndexedX(true)), // add 1 cycle if page boundary is crossed
             /* INC_abx */ 0xFE => (Op::INC, 7,  true, AddrMode::AbsoluteIndexedX(false)),
             /* ** undocumented/forbidden instructions ** */
             /* HLT     */ 0x02 => (Op::HLT, 1, false, AddrMode::Implied),
             /* SLO_izx */ 0x03 => (Op::SLO, 8,  true, AddrMode::IndexedIndirectX),
             /* NOP_zp  */ 0x04 => (Op::NOP, 3, false, AddrMode::Zeropage),
             /* SLO_zp  */ 0x07 => (Op::SLO, 5,  true, AddrMode::Zeropage),
             /* ANC_imm */ 0x0B => (Op::ANC, 2, false, AddrMode::Immediate),
             /* NOP_abs */ 0x0C => (Op::NOP, 4, false, AddrMode::Absolute),
             /* SLO_abs */ 0x0F => (Op::SLO, 6,  true, AddrMode::Absolute),
             /* HLT     */ 0x12 => (Op::HLT, 1, false, AddrMode::Implied),
             /* SLO_izy */ 0x13 => (Op::SLO, 8,  true, AddrMode::IndirectIndexedY(false)),
             /* NOP_zpx */ 0x14 => (Op::NOP, 4, false, AddrMode::ZeropageIndexedX),
             /* SLO_zpx */ 0x17 => (Op::SLO, 6,  true, AddrMode::ZeropageIndexedX),
             /* NOP     */ 0x1A => (Op::NOP, 2, false, AddrMode::Implied),
             /* SLO_aby */ 0x1B => (Op::SLO, 7,  true, AddrMode::AbsoluteIndexedY(false)),
             /* NOP_abx */ 0x1C => (Op::NOP, 5, false, AddrMode::AbsoluteIndexedX(true)), // add 1 cycle if page boudary is crossed
             /* SLO_abx */ 0x1F => (Op::SLO, 7,  true, AddrMode::AbsoluteIndexedX(false)),
             /* HLT     */ 0x22 => (Op::HLT, 1, false, AddrMode::Implied),
             /* RLA_izx */ 0x23 => (Op::RLA, 8,  true, AddrMode::IndexedIndirectX),
             /* RLA_zp  */ 0x27 => (Op::RLA, 5,  true, AddrMode::Zeropage),
             /* ANC_imm */ 0x2B => (Op::ANC, 2, false, AddrMode::Immediate),
             /* RLA_abs */ 0x2F => (Op::RLA, 6,  true, AddrMode::Absolute),
             /* HLT     */ 0x32 => (Op::HLT, 1, false, AddrMode::Implied),
             /* RLA_izy */ 0x33 => (Op::RLA, 8,  true, AddrMode::IndirectIndexedY(false)),
             /* NOP_zpx */ 0x34 => (Op::NOP, 4, false, AddrMode::ZeropageIndexedX),
             /* RLA_zpx */ 0x37 => (Op::RLA, 6,  true, AddrMode::ZeropageIndexedX),
             /* NOP     */ 0x3A => (Op::NOP, 2, false, AddrMode::Implied),
             /* RLA_aby */ 0x3B => (Op::RLA, 7,  true, AddrMode::AbsoluteIndexedY(false)),
             /* NOP_abx */ 0x3C => (Op::NOP, 5, false, AddrMode::AbsoluteIndexedX(true)), // add 1 cycle if page boundary is crossed
             /* RLA_abx */ 0x3F => (Op::RLA, 7,  true, AddrMode::AbsoluteIndexedX(false)),
             /* HLT     */ 0x42 => (Op::HLT, 1, false, AddrMode::Implied),
             /* SRE_izx */ 0x43 => (Op::SRE, 8,  true, AddrMode::IndexedIndirectX),
             /* NOP     */ 0x44 => (Op::NOP, 3, false, AddrMode::Implied),
             /* SRE_zp  */ 0x47 => (Op::SRE, 5,  true, AddrMode::Zeropage),
             /* ALR_imm */ 0x4B => (Op::ALR, 2, false, AddrMode::Immediate),
             /* SRE_abs */ 0x4F => (Op::SRE, 6,  true, AddrMode::Absolute),
             /* HLT     */ 0x52 => (Op::HLT, 1, false, AddrMode::Implied),
             /* SRE_izy */ 0x53 => (Op::SRE, 8,  true, AddrMode::IndirectIndexedY(false)),
             /* NOP_zpx */ 0x54 => (Op::NOP, 4, false, AddrMode::ZeropageIndexedX),
             /* SRE_zpx */ 0x57 => (Op::SRE, 6,  true, AddrMode::ZeropageIndexedX),
             /* NOP     */ 0x5A => (Op::NOP, 2, false, AddrMode::Implied),
             /* SRE_aby */ 0x5B => (Op::SRE, 7,  true, AddrMode::AbsoluteIndexedY(false)),
             /* NOP_abx */ 0x5C => (Op::NOP, 5, false, AddrMode::AbsoluteIndexedX(true)), // add 1 cycle if page boundary is crossed
             /* SRE_abx */ 0x5F => (Op::SRE, 7,  true, AddrMode::AbsoluteIndexedX(false)),
             /* HLT     */ 0x62 => (Op::HLT, 1, false, AddrMode::Implied),
             /* RRA_izx */ 0x63 => (Op::RRA, 8,  true, AddrMode::IndexedIndirectX),
             /* NOP_zp  */ 0x64 => (Op::NOP, 3, false, AddrMode::Zeropage),
             /* RRA_zp  */ 0x67 => (Op::RRA, 5,  true, AddrMode::Zeropage),
             /* ARR     */ 0x6B => (Op::ARR, 2, false, AddrMode::Implied),
             /* RRA_abs */ 0x6F => (Op::RRA, 6,  true, AddrMode::Absolute),
             /* HLT     */ 0x72 => (Op::HLT, 1, false, AddrMode::Implied),
             /* RRA_izy */ 0x73 => (Op::RRA, 8, false, AddrMode::IndirectIndexedY(false)),
             /* NOP_zpx */ 0x74 => (Op::NOP, 4, false, AddrMode::ZeropageIndexedX),
             /* RRA_zpx */ 0x77 => (Op::RRA, 6,  true, AddrMode::ZeropageIndexedX),
             /* NOP     */ 0x7A => (Op::NOP, 2, false, AddrMode::Implied),
             /* RRA_aby */ 0x7B => (Op::RRA, 7,  true, AddrMode::AbsoluteIndexedY(false)),
             /* NOP_abx */ 0x7C => (Op::NOP, 5, false, AddrMode::AbsoluteIndexedX(true)), // add 1 cycle if page boundary is crossed
             /* RRA_abx */ 0x7F => (Op::RRA, 7,  true, AddrMode::AbsoluteIndexedX(false)),
             /* NOP_imm */ 0x80 => (Op::NOP, 2, false, AddrMode::Immediate),
             /* NOP_imm */ 0x82 => (Op::NOP, 2, false, AddrMode::Immediate),
             /* SAX_izx */ 0x83 => (Op::SAX, 6, false, AddrMode::IndexedIndirectX),
             /* SAX_zp  */ 0x87 => (Op::SAX, 3, false, AddrMode::Zeropage),
             /* NOP_imm */ 0x89 => (Op::NOP, 2, false, AddrMode::Immediate),
             /* XAA_imm */ 0x8B => (Op::XAA, 2, false, AddrMode::Immediate),
             /* SAX_abs */ 0x8F => (Op::SAX, 4, false, AddrMode::Absolute),
             /* HLT     */ 0x92 => (Op::HLT, 1, false, AddrMode::Implied),
             /* AHX_izy */ 0x93 => (Op::AHX, 6, false, AddrMode::IndirectIndexedY(false)),
             /* SAX_zpy */ 0x97 => (Op::SAX, 4, false, AddrMode::ZeropageIndexedY),
             /* TAS_aby */ 0x9B => (Op::TAS, 5, false, AddrMode::AbsoluteIndexedY(false)),
             /* SHY_abx */ 0x9C => (Op::SHY, 5, false, AddrMode::AbsoluteIndexedX(false)),
             /* SHX_aby */ 0x9E => (Op::SHX, 5, false, AddrMode::AbsoluteIndexedY(false)),
             /* AHX_aby */ 0x9F => (Op::AHX, 5, false, AddrMode::AbsoluteIndexedY(false)),
             /* LAX_izx */ 0xA3 => (Op::LAX, 6, false, AddrMode::IndexedIndirectX),
             /* LAX_zp  */ 0xA7 => (Op::LAX, 3, false, AddrMode::Zeropage),
             /* LAX_imm */ 0xAB => (Op::LAX, 2, false, AddrMode::Immediate),
             /* LAX_abs */ 0xAF => (Op::LAX, 4, false, AddrMode::Absolute),
             /* HLT     */ 0xB2 => (Op::HLT, 1, false, AddrMode::Implied),
             /* LAX_izy */ 0xB3 => (Op::LAX, 6, false, AddrMode::IndirectIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* LAX_zpy */ 0xB7 => (Op::LAX, 4, false, AddrMode::ZeropageIndexedY),
             /* LAS_aby */ 0xBB => (Op::LAS, 5, false, AddrMode::AbsoluteIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* LAX_aby */ 0xBF => (Op::LAX, 5, false, AddrMode::AbsoluteIndexedY(true)), // add 1 cycle if page boundary is crossed
             /* NOP_imm */ 0xC2 => (Op::NOP, 2, false, AddrMode::Immediate),
             /* DCP_izx */ 0xC3 => (Op::DCP, 8,  true, AddrMode::IndexedIndirectX),
             /* DCP_zp  */ 0xC7 => (Op::DCP, 5,  true, AddrMode::Zeropage),
             /* AXS_imm */ 0xCB => (Op::AXS, 2, false, AddrMode::Immediate),
             /* DCP_abs */ 0xCF => (Op::DCP, 6,  true, AddrMode::Absolute),
             /* HLT     */ 0xD2 => (Op::HLT, 1, false, AddrMode::Implied),
             /* DCP_izy */ 0xD3 => (Op::DCP, 8,  true, AddrMode::IndirectIndexedY(false)),
             /* NOP_zpx */ 0xD4 => (Op::NOP, 4, false, AddrMode::ZeropageIndexedX),
             /* DCP_zpx */ 0xD7 => (Op::DCP, 6,  true, AddrMode::ZeropageIndexedX),
             /* NOP     */ 0xDA => (Op::NOP, 2, false, AddrMode::Implied),
             /* DCP_aby */ 0xDB => (Op::DCP, 7,  true, AddrMode::AbsoluteIndexedY(false)),
             /* NOP_abx */ 0xDC => (Op::NOP, 5, false, AddrMode::AbsoluteIndexedX(true)), // add 1 cycle if page boundary is crossed
             /* DCP_abx */ 0xDF => (Op::DCP, 7,  true, AddrMode::AbsoluteIndexedX(false)),
             /* NOP_imm */ 0xE2 => (Op::NOP, 2, false, AddrMode::Immediate),
             /* ISC_izx */ 0xE3 => (Op::ISC, 8,  true, AddrMode::IndexedIndirectX),
             /* ISC_zp  */ 0xE7 => (Op::ISC, 5,  true, AddrMode::Zeropage),
             /* SBC_imm */ 0xEB => (Op::SBC, 2, false, AddrMode::Immediate),
             /* ISC_abs */ 0xEF => (Op::ISC, 6,  true, AddrMode::Absolute),
             /* HLT     */ 0xF2 => (Op::HLT, 1, false, AddrMode::Implied),
             /* ISC_izy */ 0xF3 => (Op::ISC, 8,  true, AddrMode::IndirectIndexedY(false)),
             /* NOP_zpx */ 0xF4 => (Op::NOP, 4, false, AddrMode::ZeropageIndexedX),
             /* ISC_zpx */ 0xF7 => (Op::ISC, 6,  true, AddrMode::ZeropageIndexedX),
             /* NOP     */ 0xFA => (Op::NOP, 2, false, AddrMode::Implied),
             /* ISC_aby */ 0xFB => (Op::ISC, 7,  true, AddrMode::AbsoluteIndexedY(false)),
             /* NOP_abx */ 0xFC => (Op::NOP, 5, false, AddrMode::AbsoluteIndexedX(true)), // add 1 cycle if page boundary is crossed
             /* ISC_abx */ 0xFF => (Op::ISC, 7,  true, AddrMode::AbsoluteIndexedX(false)),
             
             _ => return None
         })
}
