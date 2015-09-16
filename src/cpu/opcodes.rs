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

#![allow(dead_code)]
#![allow(non_camel_case_types)]
use cpu;
use std::fmt;
use std::num::Wrapping;

pub enum AddrMode
{
    Implied,
    Accumulator,
    Immediate(u8),
    Absolute(u16),
    AbsoluteIndexedX(u16),
    AbsoluteIndexedY(u16),
    Zeropage(u8),
    ZeropageIndexedX(u8),
    ZeropageIndexedY(u8),
    Relative(i8),
    Indirect(u16),
    IndexedIndirectX(u8),
    IndirectIndexedY(u8)
}

impl AddrMode
{
    pub fn get_address(&self, cpu: &mut cpu::CPU) -> u16
    {
        match *self
        {
            AddrMode::Implied                     => panic!("Can't get operand address"),
            AddrMode::Accumulator                 => panic!("Can't get operand address"),
            AddrMode::Immediate(..)               => panic!("Can't get operand address"),
            AddrMode::Absolute(next_word)         => next_word,
            AddrMode::AbsoluteIndexedX(next_word) => next_word + cpu.X as u16,
            AddrMode::AbsoluteIndexedY(next_word) => next_word + cpu.Y as u16,
            AddrMode::Zeropage(next_byte)         => next_byte as u16,
            AddrMode::ZeropageIndexedX(next_byte) => (Wrapping(next_byte) + Wrapping(cpu.X)).0 as u16,
            AddrMode::ZeropageIndexedY(next_byte) => (Wrapping(next_byte) + Wrapping(cpu.Y)).0 as u16,
            AddrMode::Relative(next_byte)         => (cpu.PC as i16 + next_byte as i16) as u16,
            AddrMode::Indirect(next_word)         => cpu.mem.read_word_le(next_word),
            AddrMode::IndexedIndirectX(next_byte) => cpu.mem.read_word_le((Wrapping(next_byte) + Wrapping(cpu.X)).0 as u16),
            AddrMode::IndirectIndexedY(next_byte) => cpu.mem.read_word_le(next_byte as u16) + cpu.Y as u16
        }
    }
    
    pub fn get_value(&self, cpu: &mut cpu::CPU) -> u8
    {
        match *self
        {
            AddrMode::Implied                     => panic!("Can't get operand value"),
            AddrMode::Accumulator                 => cpu.A,
            AddrMode::Immediate(next_byte)        => next_byte,
            _ => {
                let addr = self.get_address(cpu);
                cpu.mem.read_byte(addr)
            }
        }
    }

    pub fn set_value(&self, cpu: &mut cpu::CPU, value: u8)
    {
        match *self
        {
            AddrMode::Implied         => panic!("Can't set operand value"),
            AddrMode::Accumulator     => cpu.A = value,
            AddrMode::Immediate(..)   => panic!("Can't set operand value"),
            AddrMode::Relative(..)    => panic!("Can't set operand value"),
            _ => {
                let addr = self.get_address(cpu);
                let byte_written = cpu.mem.write_byte(addr, value);
                if !byte_written
                {
                    println!("${:04X}: ROM write (0x{:02X} -> ${:04X})   A: {:02X} X: {:02X} Y: {:02X} SP: {:02X} 00: {:02X} 01: {:02X} CZIDB-VN: [{:08b}]", cpu.prev_PC - 1, value, addr, cpu.A, cpu.X, cpu.Y, cpu.SP, cpu.mem.read_byte(0x0000), cpu.mem.read_byte(0x0001), cpu.P);
                }
            }
        }
    }
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
    // Inc/dec
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

impl Op
{
    pub fn run(&self, operand: &AddrMode, cpu: &mut cpu::CPU)
    {
        match *self
        {
            Op::LDA => {
                let na = operand.get_value(cpu);
                cpu.A = na;
                cpu.set_zn_flags(na);
            },
            Op::LDX => {
                let nx = operand.get_value(cpu);
                cpu.X = nx;
                cpu.set_zn_flags(nx);
            },
            Op::LDY => {
                let ny = operand.get_value(cpu);
                cpu.Y = ny;
                cpu.set_zn_flags(ny);
            },
            Op::STA => {
                let a = cpu.A;
                operand.set_value(cpu, a);
            },
            Op::STX => {
                let x = cpu.X;
                operand.set_value(cpu, x);
            },
            Op::STY => {
                let y = cpu.Y;
                operand.set_value(cpu, y);
            },
            Op::TAX => {
                cpu.X = cpu.A;
                let x = cpu.X;
                cpu.set_zn_flags(x);
            },
            Op::TAY => {
                cpu.Y = cpu.A;
                let y = cpu.Y;
                cpu.set_zn_flags(y);
            },
            Op::TXA => {
                cpu.A = cpu.X;
                let a = cpu.A;
                cpu.set_zn_flags(a);
            },
            Op::TYA => {
                cpu.A = cpu.Y;
                let a = cpu.A;
                cpu.set_zn_flags(a);
            },
            Op::TSX => {
                cpu.X = cpu.SP;
                let x = cpu.X;
                cpu.set_zn_flags(x);
            },
            Op::TXS => {
                cpu.SP = cpu.X;
            },
            Op::PHA => {
                let a = cpu.A;
                cpu.push_byte(a);
            },
            Op::PHP => {
                let p = cpu.P;
                cpu.push_byte(p);
            },
            Op::PLA => {
                let a = cpu.pop_byte();
                cpu.A = a;
                cpu.set_zn_flags(a);
            },
            Op::PLP => {
                let p = cpu.pop_byte();
                cpu.P = p;
                // PLP may affect even the unused flag bit
                cpu.P |= 0x20;
            },
            Op::AND => {
                let v = operand.get_value(cpu);
                let na = cpu.A & v;
                cpu.A = na;
                cpu.set_zn_flags(na);
                
            },
            Op::EOR => {
                let v = operand.get_value(cpu);
                let na = cpu.A ^ v;
                cpu.A = na;
                cpu.set_zn_flags(na);
            },
            Op::ORA => {
                let v = operand.get_value(cpu);
                let na = cpu.A | v;
                cpu.A = na;
                cpu.set_zn_flags(na);
            },
            Op::BIT => {
                let v = operand.get_value(cpu);
                let a = cpu.A;
                cpu.set_status_flag(cpu::StatusFlag::Negative, (v & 0x80) != 0);
                cpu.set_status_flag(cpu::StatusFlag::Overflow, (v & 0x40) != 0);
                cpu.set_status_flag(cpu::StatusFlag::Zero,     (v & a)    == 0);
            },
            Op::ADC => {
                // TODO: support decimal mode
                let v = operand.get_value(cpu);
                let mut res: u16 = cpu.A as u16 + v as u16;
                if cpu.get_status_flag(cpu::StatusFlag::Carry)
                {
                    res += 0x0001;
                }
                cpu.set_status_flag(cpu::StatusFlag::Carry, (res & 0x0100) != 0);
                let res = res as u8;
                let is_overflow = (cpu.A ^ res) & 0x80 != 0 && (cpu.A ^ v) & 0x80 == 0x80;
                cpu.set_status_flag(cpu::StatusFlag::Overflow, is_overflow);
		cpu.A = res;
		cpu.set_zn_flags(res);
            },
            Op::SBC => {
                // TODO: support decimal mode
                let v = operand.get_value(cpu);
                let mut res: u16 = cpu.A as u16 - v as u16;
                if !cpu.get_status_flag(cpu::StatusFlag::Carry)
                {
                    res -= 0x0001;
                }
                cpu.set_status_flag(cpu::StatusFlag::Carry, (res & 0x0100) == 0);
                let res = res as u8;
                let is_overflow = (cpu.A ^ res) & 0x80 != 0 && (cpu.A ^ v) & 0x80 == 0x80;
                cpu.set_status_flag(cpu::StatusFlag::Overflow, is_overflow);
		cpu.A = res;
		cpu.set_zn_flags(res);
            },
            Op::CMP => {
	        let res = cpu.A as i16 - operand.get_value(cpu) as i16;
		cpu.set_status_flag(cpu::StatusFlag::Carry, res >= 0);
		cpu.set_zn_flags(res as u8);
            },
            Op::CPX => {
	        let res = cpu.X as i16 - operand.get_value(cpu) as i16;
		cpu.set_status_flag(cpu::StatusFlag::Carry, res >= 0);
		cpu.set_zn_flags(res as u8);
            },
            Op::CPY => {
	        let res = cpu.Y as i16 - operand.get_value(cpu) as i16;
		cpu.set_status_flag(cpu::StatusFlag::Carry, res >= 0);
		cpu.set_zn_flags(res as u8);
            },
            Op::INC => {
                let v = (Wrapping(operand.get_value(cpu)) + Wrapping(0x01)).0;
                operand.set_value(cpu, v);
                cpu.set_zn_flags(v);
            },
            Op::INX => {
                cpu.X = (Wrapping(cpu.X) + Wrapping(0x01)).0;
                let x = cpu.X;
                cpu.set_zn_flags(x);
            },
            Op::INY => {
                cpu.Y = (Wrapping(cpu.Y) + Wrapping(0x01)).0;
                let y = cpu.Y;
                cpu.set_zn_flags(y);
            },
            Op::DEC => {
                let v = (Wrapping(operand.get_value(cpu)) - Wrapping(0x01)).0;
                operand.set_value(cpu, v);
                cpu.set_zn_flags(v);
            },
            Op::DEX => {
                cpu.X = (Wrapping(cpu.X) - Wrapping(0x01)).0;
                let x = cpu.X;
                cpu.set_zn_flags(x);
            },
            Op::DEY => {
                cpu.Y = (Wrapping(cpu.Y) - Wrapping(0x01)).0;
                let y = cpu.Y;
                cpu.set_zn_flags(y);
            },
            Op::ASL => {
                let v = operand.get_value(cpu);
                cpu.set_status_flag(cpu::StatusFlag::Carry, (v & 0x80) != 0);
                let res = v << 1;
                operand.set_value(cpu, res);
                cpu.set_zn_flags(res);
            },
            Op::LSR => {
                let v = operand.get_value(cpu);
                cpu.set_status_flag(cpu::StatusFlag::Carry, (v & 0x80) != 0);
                let res = v >> 1;
                operand.set_value(cpu, res);
                cpu.set_zn_flags(res);
            },
            Op::ROL => {
                let c = cpu.get_status_flag(cpu::StatusFlag::Carry);
                let v = operand.get_value(cpu);
                cpu.set_status_flag(cpu::StatusFlag::Carry, (v & 0x80) != 0);
                let mut res = v << 1;
                if c
                {
                    res |= 0x01;
                }                                
                operand.set_value(cpu, res);
                cpu.set_zn_flags(res);
            },
            Op::ROR => {
                let c = cpu.get_status_flag(cpu::StatusFlag::Carry);
                let v = operand.get_value(cpu);
                cpu.set_status_flag(cpu::StatusFlag::Carry, (v & 0x01) != 0);
                let mut res = v >> 1;
                if c
                {
                    res |= 0x80;
                }                                
                operand.set_value(cpu, res);
                cpu.set_zn_flags(res);
            },
            Op::JMP => {
                let npc = operand.get_address(cpu);
                cpu.PC = npc;
            },
            Op::JSR => {
                let npc = operand.get_address(cpu);
                let pc = cpu.PC - 0x0001;
                cpu.push_word(pc);
                cpu.PC = npc;
            },
            Op::RTS => {
                let pc = cpu.pop_word();
                cpu.PC = pc + 0x0001;
            },
            Op::BCC => {
                let npc = operand.get_address(cpu);
                if !cpu.get_status_flag(cpu::StatusFlag::Carry)
                {
                    cpu.PC = npc;
                }
            },
            Op::BCS => {
                let npc = operand.get_address(cpu);
                if cpu.get_status_flag(cpu::StatusFlag::Carry)
                {
                    cpu.PC = npc;
                }
            },
            Op::BEQ => {
                let npc = operand.get_address(cpu);
                if cpu.get_status_flag(cpu::StatusFlag::Zero)
                {
                    cpu.PC = npc;
                }
            },
            Op::BMI => {
                let npc = operand.get_address(cpu);
                if cpu.get_status_flag(cpu::StatusFlag::Negative)
                {
                    cpu.PC = npc;
                }
            },
            Op::BNE => {
                let npc = operand.get_address(cpu);
                if !cpu.get_status_flag(cpu::StatusFlag::Zero)
                {
                    cpu.PC = npc;
                }
            },
            Op::BPL => {
                let npc = operand.get_address(cpu);
                if !cpu.get_status_flag(cpu::StatusFlag::Negative)
                {
                    cpu.PC = npc;
                }
            },
            Op::BVC => {
                let npc = operand.get_address(cpu);
                if !cpu.get_status_flag(cpu::StatusFlag::Overflow)
                {
                    cpu.PC = npc;
                }
            },
            Op::BVS => {
                let npc = operand.get_address(cpu);
                if cpu.get_status_flag(cpu::StatusFlag::Overflow)
                {
                    cpu.PC = npc;
                }
            },
            Op::CLC => {
                cpu.set_status_flag(cpu::StatusFlag::Carry, false);
            },
            Op::CLD => {
                cpu.set_status_flag(cpu::StatusFlag::DecimalMode, false);
            },
            Op::CLI => {
                cpu.set_status_flag(cpu::StatusFlag::InterruptDisable, false);
            },
            Op::CLV => {
                cpu.set_status_flag(cpu::StatusFlag::Overflow, false);
            },
            Op::SEC => {
                cpu.set_status_flag(cpu::StatusFlag::Carry, true);
            },
            Op::SED => {
                cpu.set_status_flag(cpu::StatusFlag::DecimalMode, true);
            },
            Op::SEI => {
                cpu.set_status_flag(cpu::StatusFlag::InterruptDisable, true);
            },
            Op::BRK => {
                cpu.set_status_flag(cpu::StatusFlag::Break, true);
                let pc = cpu.PC + 0x0002;
                let p  = cpu.P;
                cpu.push_word(pc);
                cpu.push_byte(p);
                cpu.PC = cpu.mem.read_word_le(cpu::IRQ_VECTOR);
                cpu.set_status_flag(cpu::StatusFlag::InterruptDisable, true);
            },
            Op::NOP => (),
            Op::RTI => {
                let p = cpu.pop_byte();
                let pc = cpu.pop_word();
                cpu.P = p;
                cpu.PC = pc;
                cpu.P |= 0x20;
            },
            Op::HLT => panic!("Received HLT instruction at ${:04X}", cpu.PC),
            _       => () //println!("Unknown op: {}{} at ${:04X}", self, addr_mode, cpu.PC)
        }
    }
}

// debug display for opcodes
impl fmt::Display for Op
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let op_name = match *self {
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

pub fn get_instruction(opcode: u8, cpu: &mut cpu::CPU) -> Option<(Op, u8, AddrMode)>
{
    Some(match opcode
         {
             /* ** documented instructions ** */
             /* BRK     */ 0x00 => (Op::BRK, 7, AddrMode::Implied),
             /* ORA_izx */ 0x01 => (Op::ORA, 6, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* ORA_zp  */ 0x05 => (Op::ORA, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* ASL_zp  */ 0x06 => (Op::ASL, 5, AddrMode::Zeropage(cpu.next_byte())), 
             /* PHP     */ 0x08 => (Op::PHP, 3, AddrMode::Implied),
             /* ORA_imm */ 0x09 => (Op::ORA, 2, AddrMode::Immediate(cpu.next_byte())),
             /* ASL     */ 0x0A => (Op::ASL, 2, AddrMode::Accumulator),
             /* ORA_abs */ 0x0D => (Op::ORA, 4, AddrMode::Absolute(cpu.next_word())),
             /* ASL_abs */ 0x0E => (Op::ASL, 6, AddrMode::Absolute(cpu.next_word())),
             /* BPL_rel */ 0x10 => (Op::BPL, 2, AddrMode::Relative(cpu.next_byte() as i8)), // add 1 cycle if page boundary is crossed
             /* ORA_izy */ 0x11 => (Op::ORA, 5, AddrMode::IndirectIndexedY(cpu.next_byte())), // add 1 cycle if page boundary is crossed
             /* ORA_zpx */ 0x15 => (Op::ORA, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* ASL_zpx */ 0x16 => (Op::ASL, 6, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* CLC     */ 0x18 => (Op::CLC, 2, AddrMode::Implied),
             /* ORA_aby */ 0x19 => (Op::ORA, 4, AddrMode::AbsoluteIndexedY(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* ORA_abx */ 0x1D => (Op::ORA, 4, AddrMode::AbsoluteIndexedX(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* ASL_abx */ 0x1E => (Op::ASL, 7, AddrMode::AbsoluteIndexedX(cpu.next_word())),
             /* JSR_abs */ 0x20 => (Op::JSR, 6, AddrMode::Absolute(cpu.next_word())),
             /* AND_izx */ 0x21 => (Op::AND, 6, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* BIT_zp  */ 0x24 => (Op::BIT, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* AND_zp  */ 0x25 => (Op::AND, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* ROL_zp  */ 0x26 => (Op::ROL, 5, AddrMode::Zeropage(cpu.next_byte())),
             /* PLP     */ 0x28 => (Op::PLP, 4, AddrMode::Implied),
             /* AND_imm */ 0x29 => (Op::AND, 2, AddrMode::Immediate(cpu.next_byte())),
             /* ROL     */ 0x2A => (Op::ROL, 2, AddrMode::Accumulator),
             /* BIT_abs */ 0x2C => (Op::BIT, 4, AddrMode::Absolute(cpu.next_word())),
             /* AND_abs */ 0x2D => (Op::AND, 4, AddrMode::Absolute(cpu.next_word())),
             /* ROL_abs */ 0x2E => (Op::ROL, 6, AddrMode::Absolute(cpu.next_word())),
             /* BMI_rel */ 0x30 => (Op::BMI, 2, AddrMode::Relative(cpu.next_byte() as i8)), // add 1 cycle if page boundary is crossed
             /* AND_izy */ 0x31 => (Op::AND, 5, AddrMode::IndirectIndexedY(cpu.next_byte())), // add 1 cycle if page boundary is crossed
             /* AND_zpx */ 0x35 => (Op::AND, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* ROL_zpx */ 0x36 => (Op::ROL, 6, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* SEC     */ 0x38 => (Op::SEC, 2, AddrMode::Implied),
             /* AND_aby */ 0x39 => (Op::AND, 4, AddrMode::AbsoluteIndexedY(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* AND_abx */ 0x3D => (Op::AND, 4, AddrMode::AbsoluteIndexedX(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* ROL_abx */ 0x3E => (Op::ROL, 7, AddrMode::AbsoluteIndexedX(cpu.next_word())),
             /* RTI     */ 0x40 => (Op::RTI, 6, AddrMode::Implied),
             /* EOR_izx */ 0x41 => (Op::EOR, 6, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* EOR_zp  */ 0x45 => (Op::EOR, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* LSR_zp  */ 0x46 => (Op::LSR, 5, AddrMode::Zeropage(cpu.next_byte())),
             /* PHA     */ 0x48 => (Op::PHA, 3, AddrMode::Implied),
             /* EOR_imm */ 0x49 => (Op::EOR, 2, AddrMode::Immediate(cpu.next_byte())),
             /* LSR     */ 0x4A => (Op::LSR, 2, AddrMode::Accumulator),
             /* JMP_abs */ 0x4C => (Op::JMP, 3, AddrMode::Absolute(cpu.next_word())),
             /* EOR_abs */ 0x4D => (Op::EOR, 4, AddrMode::Absolute(cpu.next_word())),
             /* LSR_abs */ 0x4E => (Op::LSR, 6, AddrMode::Absolute(cpu.next_word())),
             /* BVC_rel */ 0x50 => (Op::BVC, 2, AddrMode::Relative(cpu.next_byte() as i8)), // add 1 cycle if page boundary is crossed
             /* EOR_izy */ 0x51 => (Op::EOR, 5, AddrMode::IndirectIndexedY(cpu.next_byte())), // add 1 cycle if page boundary is crossed
             /* EOR_zpx */ 0x55 => (Op::EOR, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* LSR_zpx */ 0x56 => (Op::LSR, 6, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* CLI     */ 0x58 => (Op::CLI, 2, AddrMode::Implied),
             /* EOR_aby */ 0x59 => (Op::EOR, 4, AddrMode::AbsoluteIndexedY(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* EOR_abx */ 0x5D => (Op::EOR, 4, AddrMode::AbsoluteIndexedX(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* LSR_abx */ 0x5E => (Op::LSR, 7, AddrMode::AbsoluteIndexedX(cpu.next_word())),
             /* RTS     */ 0x60 => (Op::RTS, 6, AddrMode::Implied),
             /* ADC_izx */ 0x61 => (Op::ADC, 6, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* ADC_zp  */ 0x65 => (Op::ADC, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* ROR_zp  */ 0x66 => (Op::ROR, 5, AddrMode::Zeropage(cpu.next_byte())),
             /* PLA     */ 0x68 => (Op::PLA, 4, AddrMode::Implied),
             /* ADC_imm */ 0x69 => (Op::ADC, 2, AddrMode::Immediate(cpu.next_byte())),
             /* ROR     */ 0x6A => (Op::ROR, 2, AddrMode::Accumulator),
             /* JMP_ind */ 0x6C => (Op::JMP, 5, AddrMode::Indirect(cpu.next_word())),
             /* ADC_abs */ 0x6D => (Op::ADC, 4, AddrMode::Absolute(cpu.next_word())),
             /* ROR_abs */ 0x6E => (Op::ROR, 6, AddrMode::Absolute(cpu.next_word())),
             /* BVS_rel */ 0x70 => (Op::BVS, 2, AddrMode::Relative(cpu.next_byte() as i8)), // add 1 cycle if page boundary is crossed
             /* ADC_izy */ 0x71 => (Op::ADC, 5, AddrMode::IndirectIndexedY(cpu.next_byte())), // add 1 cycle if page boundary is crossed
             /* ADC_zpx */ 0x75 => (Op::ADC, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* ROR_zpx */ 0x76 => (Op::ROR, 6, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* SEI     */ 0x78 => (Op::SEI, 2, AddrMode::Implied),
             /* ADC_aby */ 0x79 => (Op::ADC, 4, AddrMode::AbsoluteIndexedY(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* ADC_abx */ 0x7D => (Op::ADC, 4, AddrMode::AbsoluteIndexedX(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* ROR_abx */ 0x7E => (Op::ROR, 7, AddrMode::AbsoluteIndexedX(cpu.next_word())),
             /* STA_izx */ 0x81 => (Op::STA, 6, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* STY_zp  */ 0x84 => (Op::STY, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* STA_zp  */ 0x85 => (Op::STA, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* STX_zp  */ 0x86 => (Op::STX, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* DEY     */ 0x88 => (Op::DEY, 2, AddrMode::Implied),
             /* TXA     */ 0x8A => (Op::TXA, 2, AddrMode::Implied),
             /* STY_abs */ 0x8C => (Op::STY, 4, AddrMode::Absolute(cpu.next_word())),
             /* STA_abs */ 0x8D => (Op::STA, 4, AddrMode::Absolute(cpu.next_word())),
             /* STX_abs */ 0x8E => (Op::STX, 4, AddrMode::Absolute(cpu.next_word())),
             /* BCC_rel */ 0x90 => (Op::BCC, 2, AddrMode::Relative(cpu.next_byte() as i8)), // add 1 cycle if page boundary is crossed
             /* STA_izy */ 0x91 => (Op::STA, 6, AddrMode::IndirectIndexedY(cpu.next_byte())),
             /* STY_zpx */ 0x94 => (Op::STY, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* STA_zpx */ 0x95 => (Op::STA, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* STX_zpy */ 0x96 => (Op::STX, 4, AddrMode::ZeropageIndexedY(cpu.next_byte())),
             /* TYA     */ 0x98 => (Op::TYA, 2, AddrMode::Implied),
             /* STA_aby */ 0x99 => (Op::STA, 5, AddrMode::AbsoluteIndexedY(cpu.next_word())),
             /* TXS     */ 0x9A => (Op::TXS, 2, AddrMode::Implied),
             /* STA_abx */ 0x9D => (Op::STA, 5, AddrMode::AbsoluteIndexedX(cpu.next_word())),
             /* LDY_imm */ 0xA0 => (Op::LDY, 2, AddrMode::Immediate(cpu.next_byte())),
             /* LDA_izx */ 0xA1 => (Op::LDA, 6, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* LDX_imm */ 0xA2 => (Op::LDX, 2, AddrMode::Immediate(cpu.next_byte())),
             /* LDY_zp  */ 0xA4 => (Op::LDY, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* LDA_zp  */ 0xA5 => (Op::LDA, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* LDX_zp  */ 0xA6 => (Op::LDX, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* TAY     */ 0xA8 => (Op::TAY, 2, AddrMode::Implied),
             /* LDA_imm */ 0xA9 => (Op::LDA, 2, AddrMode::Immediate(cpu.next_byte())),
             /* TAX     */ 0xAA => (Op::TAX, 2, AddrMode::Implied),
             /* LDY_abs */ 0xAC => (Op::LDY, 4, AddrMode::Absolute(cpu.next_word())),
             /* LDA_abs */ 0xAD => (Op::LDA, 4, AddrMode::Absolute(cpu.next_word())),
             /* LDX_abs */ 0xAE => (Op::LDX, 4, AddrMode::Absolute(cpu.next_word())),
             /* BCS_rel */ 0xB0 => (Op::BCS, 2, AddrMode::Relative(cpu.next_byte() as i8)), // add 1 cycle if page boundary is crossed
             /* LDA_izy */ 0xB1 => (Op::LDA, 5, AddrMode::IndirectIndexedY(cpu.next_byte())), // add 1 cycle if page boundary is crossed
             /* LDY_zpx */ 0xB4 => (Op::LDY, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* LDA_zpx */ 0xB5 => (Op::LDA, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* LDX_zpy */ 0xB6 => (Op::LDX, 4, AddrMode::ZeropageIndexedY(cpu.next_byte())),
             /* CLV     */ 0xB8 => (Op::CLV, 2, AddrMode::Implied),
             /* LDA_aby */ 0xB9 => (Op::LDA, 4, AddrMode::AbsoluteIndexedY(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* TSX     */ 0xBA => (Op::TSX, 2, AddrMode::Implied),
             /* LDY_abx */ 0xBC => (Op::LDY, 4, AddrMode::AbsoluteIndexedX(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* LDA_abx */ 0xBD => (Op::LDA, 4, AddrMode::AbsoluteIndexedX(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* LDX_aby */ 0xBE => (Op::LDX, 4, AddrMode::AbsoluteIndexedY(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* CPY_imm */ 0xC0 => (Op::CPY, 2, AddrMode::Immediate(cpu.next_byte())),
             /* CMP_izx */ 0xC1 => (Op::CMP, 6, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* CPY_zp  */ 0xC4 => (Op::CPY, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* CMP_zp  */ 0xC5 => (Op::CMP, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* DEC_zp  */ 0xC6 => (Op::DEC, 5, AddrMode::Zeropage(cpu.next_byte())),
             /* INY     */ 0xC8 => (Op::INY, 2, AddrMode::Implied),
             /* CMP_imm */ 0xC9 => (Op::CMP, 2, AddrMode::Immediate(cpu.next_byte())),
             /* DEX     */ 0xCA => (Op::DEX, 2, AddrMode::Implied),
             /* CPY_abs */ 0xCC => (Op::CPY, 4, AddrMode::Absolute(cpu.next_word())),
             /* CMP_abs */ 0xCD => (Op::CMP, 4, AddrMode::Absolute(cpu.next_word())),
             /* DEC_abs */ 0xCE => (Op::DEC, 6, AddrMode::Absolute(cpu.next_word())),
             /* BNE_rel */ 0xD0 => (Op::BNE, 2, AddrMode::Relative(cpu.next_byte() as i8)), // add 1 cycle if page boundary is crossed
             /* CMP_izy */ 0xD1 => (Op::CMP, 5, AddrMode::IndirectIndexedY(cpu.next_byte())), // add 1 cycle if page boundary is crossed
             /* CMP_zpx */ 0xD5 => (Op::CMP, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* DEC_zpx */ 0xD6 => (Op::DEC, 6, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* CLD     */ 0xD8 => (Op::CLD, 2, AddrMode::Implied),
             /* CMP_aby */ 0xD9 => (Op::CMP, 4, AddrMode::AbsoluteIndexedY(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* CMP_abx */ 0xDD => (Op::CMP, 4, AddrMode::AbsoluteIndexedX(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* DEC_abx */ 0xDE => (Op::DEC, 7, AddrMode::AbsoluteIndexedX(cpu.next_word())),
             /* CPX_imm */ 0xE0 => (Op::CPX, 2, AddrMode::Immediate(cpu.next_byte())),
             /* SBC_izx */ 0xE1 => (Op::SBC, 6, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* CPX_zp  */ 0xE4 => (Op::CPX, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* SBC_zp  */ 0xE5 => (Op::SBC, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* INC_zp  */ 0xE6 => (Op::INC, 5, AddrMode::Zeropage(cpu.next_byte())),
             /* INX     */ 0xE8 => (Op::INX, 2, AddrMode::Implied),
             /* SBC_imm */ 0xE9 => (Op::SBC, 2, AddrMode::Immediate(cpu.next_byte())),
             /* NOP     */ 0xEA => (Op::NOP, 2, AddrMode::Implied),
             /* CPX     */ 0xEC => (Op::CPX, 4, AddrMode::Implied),
             /* SBC_abs */ 0xED => (Op::SBC, 4, AddrMode::Absolute(cpu.next_word())),
             /* INC_abs */ 0xEE => (Op::INC, 6, AddrMode::Absolute(cpu.next_word())),
             /* BEQ_rel */ 0xF0 => (Op::BEQ, 2, AddrMode::Relative(cpu.next_byte() as i8)), // add 1 cycle if page boundary is crossed
             /* SBC_izy */ 0xF1 => (Op::SBC, 5, AddrMode::IndirectIndexedY(cpu.next_byte())), // add 1 cycle if page boundary is crossed
             /* SBC_zpx */ 0xF5 => (Op::SBC, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* INC_zpx */ 0xF6 => (Op::INC, 6, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* SED     */ 0xF8 => (Op::SED, 2, AddrMode::Implied),
             /* SBC_aby */ 0xF9 => (Op::SBC, 4, AddrMode::AbsoluteIndexedY(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* SBC_abx */ 0xFD => (Op::SBC, 4, AddrMode::AbsoluteIndexedX(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* INC_abx */ 0xFE => (Op::INC, 7, AddrMode::AbsoluteIndexedX(cpu.next_word())),
             /* ** undocumented/forbidden instructions ** */
             /* HLT     */ 0x02 => (Op::HLT, 1, AddrMode::Implied),
             /* SLO_izx */ 0x03 => (Op::SLO, 8, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* NOP_zp  */ 0x04 => (Op::NOP, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* SLO_zp  */ 0x07 => (Op::SLO, 5, AddrMode::Zeropage(cpu.next_byte())),
             /* ANC_imm */ 0x0B => (Op::ANC, 2, AddrMode::Immediate(cpu.next_byte())),
             /* NOP_abs */ 0x0C => (Op::NOP, 4, AddrMode::Absolute(cpu.next_word())),
             /* SLO_abs */ 0x0F => (Op::SLO, 6, AddrMode::Absolute(cpu.next_word())),
             /* HLT     */ 0x12 => (Op::HLT, 1, AddrMode::Implied),
             /* SLO_izy */ 0x13 => (Op::SLO, 8, AddrMode::IndirectIndexedY(cpu.next_byte())),
             /* NOP_zpx */ 0x14 => (Op::NOP, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* SLO_zpx */ 0x17 => (Op::SLO, 6, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* NOP     */ 0x1A => (Op::NOP, 2, AddrMode::Implied),
             /* SLO_aby */ 0x1B => (Op::SLO, 7, AddrMode::AbsoluteIndexedY(cpu.next_word())),
             /* NOP_abx */ 0x1C => (Op::NOP, 4, AddrMode::AbsoluteIndexedX(cpu.next_word())), // add 1 cycle if page boudary is crossed
             /* SLO_abx */ 0x1F => (Op::SLO, 7, AddrMode::AbsoluteIndexedX(cpu.next_word())),
             /* HLT     */ 0x22 => (Op::HLT, 1, AddrMode::Implied),
             /* RLA_izx */ 0x23 => (Op::RLA, 8, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* RLA_zp  */ 0x27 => (Op::RLA, 5, AddrMode::Zeropage(cpu.next_byte())),
             /* ANC_imm */ 0x2B => (Op::ANC, 2, AddrMode::Immediate(cpu.next_byte())),
             /* RLA_abs */ 0x2F => (Op::RLA, 6, AddrMode::Absolute(cpu.next_word())),
             /* HLT     */ 0x32 => (Op::HLT, 1, AddrMode::Implied),
             /* RLA_izy */ 0x33 => (Op::RLA, 8, AddrMode::IndirectIndexedY(cpu.next_byte())),
             /* NOP_zpx */ 0x34 => (Op::NOP, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* RLA_zpx */ 0x37 => (Op::RLA, 6, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* NOP     */ 0x3A => (Op::NOP, 2, AddrMode::Implied),
             /* RLA_aby */ 0x3B => (Op::RLA, 7, AddrMode::AbsoluteIndexedY(cpu.next_word())),
             /* NOP_abx */ 0x3C => (Op::NOP, 4, AddrMode::AbsoluteIndexedX(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* RLA_abx */ 0x3F => (Op::RLA, 7, AddrMode::AbsoluteIndexedX(cpu.next_word())),
             /* HLT     */ 0x42 => (Op::HLT, 1, AddrMode::Implied),
             /* SRE_izx */ 0x43 => (Op::SRE, 8, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* NOP     */ 0x44 => (Op::NOP, 3, AddrMode::Implied),
             /* SRE_zp  */ 0x47 => (Op::SRE, 5, AddrMode::Zeropage(cpu.next_byte())),
             /* ALR_imm */ 0x4B => (Op::ALR, 2, AddrMode::Immediate(cpu.next_byte())),
             /* SRE_abs */ 0x4F => (Op::SRE, 6, AddrMode::Absolute(cpu.next_word())),
             /* HLT     */ 0x52 => (Op::HLT, 1, AddrMode::Implied),
             /* SRE_izy */ 0x53 => (Op::SRE, 8, AddrMode::IndirectIndexedY(cpu.next_byte())),
             /* NOP_zpx */ 0x54 => (Op::NOP, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* SRE_zpx */ 0x57 => (Op::SRE, 6, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* NOP     */ 0x5A => (Op::NOP, 2, AddrMode::Implied),
             /* SRE_aby */ 0x5B => (Op::SRE, 7, AddrMode::AbsoluteIndexedY(cpu.next_word())),
             /* NOP_abx */ 0x5C => (Op::NOP, 4, AddrMode::AbsoluteIndexedX(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* SRE_abx */ 0x5F => (Op::SRE, 7, AddrMode::AbsoluteIndexedX(cpu.next_word())),
             /* HLT     */ 0x62 => (Op::HLT, 1, AddrMode::Implied),
             /* RRA_izx */ 0x63 => (Op::RRA, 8, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* NOP_zp  */ 0x64 => (Op::NOP, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* RRA_zp  */ 0x67 => (Op::RRA, 5, AddrMode::Zeropage(cpu.next_byte())),
             /* ARR     */ 0x6B => (Op::ARR, 2, AddrMode::Implied),
             /* RRA_abs */ 0x6F => (Op::RRA, 6, AddrMode::Absolute(cpu.next_word())),
             /* HLT     */ 0x72 => (Op::HLT, 1, AddrMode::Implied),
             /* RRA_izy */ 0x73 => (Op::RRA, 8, AddrMode::IndirectIndexedY(cpu.next_byte())),
             /* NOP_zpx */ 0x74 => (Op::NOP, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* RRA_zpx */ 0x77 => (Op::RRA, 6, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* NOP     */ 0x7A => (Op::NOP, 2, AddrMode::Implied),
             /* RRA_aby */ 0x7B => (Op::RRA, 7, AddrMode::AbsoluteIndexedY(cpu.next_word())),
             /* NOP_abx */ 0x7C => (Op::NOP, 4, AddrMode::AbsoluteIndexedX(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* RRA_abx */ 0x7F => (Op::RRA, 7, AddrMode::AbsoluteIndexedX(cpu.next_word())),
             /* NOP_imm */ 0x80 => (Op::NOP, 2, AddrMode::Immediate(cpu.next_byte())),
             /* NOP_imm */ 0x82 => (Op::NOP, 2, AddrMode::Immediate(cpu.next_byte())),
             /* SAX_izx */ 0x83 => (Op::SAX, 6, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* SAX_zp  */ 0x87 => (Op::SAX, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* NOP_imm */ 0x89 => (Op::NOP, 2, AddrMode::Immediate(cpu.next_byte())),
             /* XAA_imm */ 0x8B => (Op::XAA, 2, AddrMode::Immediate(cpu.next_byte())),
             /* SAX_abs */ 0x8F => (Op::SAX, 4, AddrMode::Absolute(cpu.next_word())),
             /* HLT     */ 0x92 => (Op::HLT, 1, AddrMode::Implied),
             /* AHX_izy */ 0x93 => (Op::AHX, 6, AddrMode::IndirectIndexedY(cpu.next_byte())),
             /* SAX_zpy */ 0x97 => (Op::SAX, 4, AddrMode::ZeropageIndexedY(cpu.next_byte())),
             /* TAS_aby */ 0x9B => (Op::TAS, 5, AddrMode::AbsoluteIndexedY(cpu.next_word())),
             /* SHY_abx */ 0x9C => (Op::SHY, 5, AddrMode::AbsoluteIndexedX(cpu.next_word())),
             /* SHX_aby */ 0x9E => (Op::SHX, 5, AddrMode::AbsoluteIndexedY(cpu.next_word())),
             /* AHX_aby */ 0x9F => (Op::AHX, 5, AddrMode::AbsoluteIndexedY(cpu.next_word())),
             /* LAX_izx */ 0xA3 => (Op::LAX, 6, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* LAX_zp  */ 0xA7 => (Op::LAX, 3, AddrMode::Zeropage(cpu.next_byte())),
             /* LAX_imm */ 0xAB => (Op::LAX, 2, AddrMode::Immediate(cpu.next_byte())),
             /* LAX_abs */ 0xAF => (Op::LAX, 4, AddrMode::Absolute(cpu.next_word())),
             /* HLT     */ 0xB2 => (Op::HLT, 1, AddrMode::Implied),
             /* LAX_izy */ 0xB3 => (Op::LAX, 5, AddrMode::IndirectIndexedY(cpu.next_byte())), // add 1 cycle if page boundary is crossed
             /* LAX_zpy */ 0xB7 => (Op::LAX, 4, AddrMode::ZeropageIndexedY(cpu.next_byte())),
             /* LAS_aby */ 0xBB => (Op::LAS, 4, AddrMode::AbsoluteIndexedY(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* LAX_aby */ 0xBF => (Op::LAX, 4, AddrMode::AbsoluteIndexedY(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* NOP_imm */ 0xC2 => (Op::NOP, 2, AddrMode::Immediate(cpu.next_byte())),
             /* DCP_izx */ 0xC3 => (Op::DCP, 8, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* DCP_zp  */ 0xC7 => (Op::DCP, 5, AddrMode::Zeropage(cpu.next_byte())),
             /* AXS_imm */ 0xCB => (Op::AXS, 2, AddrMode::Immediate(cpu.next_byte())),
             /* DCP_abs */ 0xCF => (Op::DCP, 6, AddrMode::Absolute(cpu.next_word())),
             /* HLT     */ 0xD2 => (Op::HLT, 1, AddrMode::Implied),
             /* DCP_izy */ 0xD3 => (Op::DCP, 8, AddrMode::IndirectIndexedY(cpu.next_byte())),
             /* NOP_zpx */ 0xD4 => (Op::NOP, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* DCP_zpx */ 0xD7 => (Op::DCP, 6, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* NOP     */ 0xDA => (Op::NOP, 2, AddrMode::Implied),
             /* DCP_aby */ 0xDB => (Op::DCP, 7, AddrMode::AbsoluteIndexedY(cpu.next_word())),
             /* NOP_abx */ 0xDC => (Op::NOP, 4, AddrMode::AbsoluteIndexedX(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* DCP_abx */ 0xDF => (Op::DCP, 7, AddrMode::AbsoluteIndexedX(cpu.next_word())),
             /* NOP_imm */ 0xE2 => (Op::NOP, 2, AddrMode::Immediate(cpu.next_byte())),
             /* ISC_izx */ 0xE3 => (Op::ISC, 8, AddrMode::IndexedIndirectX(cpu.next_byte())),
             /* ISC_zp  */ 0xE7 => (Op::ISC, 5, AddrMode::Zeropage(cpu.next_byte())),
             /* SBC_imm */ 0xEB => (Op::SBC, 2, AddrMode::Immediate(cpu.next_byte())),
             /* ISC_abs */ 0xEF => (Op::ISC, 6, AddrMode::Absolute(cpu.next_word())),
             /* HLT     */ 0xF2 => (Op::HLT, 1, AddrMode::Implied),
             /* ISC_izy */ 0xF3 => (Op::ISC, 8, AddrMode::IndirectIndexedY(cpu.next_byte())),
             /* NOP_zpx */ 0xF4 => (Op::NOP, 4, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* ISC_zpx */ 0xF7 => (Op::ISC, 6, AddrMode::ZeropageIndexedX(cpu.next_byte())),
             /* NOP     */ 0xFA => (Op::NOP, 2, AddrMode::Implied),
             /* ISC_aby */ 0xFB => (Op::ISC, 7, AddrMode::AbsoluteIndexedY(cpu.next_word())),
             /* NOP_abx */ 0xFC => (Op::NOP, 4, AddrMode::AbsoluteIndexedX(cpu.next_word())), // add 1 cycle if page boundary is crossed
             /* ISC_abx */ 0xFF => (Op::ISC, 7, AddrMode::AbsoluteIndexedX(cpu.next_word())),
             
             _ => return None
         })
}
