// The CPU
#![allow(dead_code)]
#![allow(non_snake_case)]
mod opcodes;
use std::mem;

// status flags for P register
enum StatusFlag
{
    N = 1 << 0, // negative flag
    V = 1 << 1, // overflow flag
    X = 1 << 2, // unused flag
    B = 1 << 3, // break flag
    D = 1 << 4, // decimal mode flag
    I = 1 << 5, // interrupt disable flag
    Z = 1 << 6, // zero flag
    C = 1 << 7  // carry flag
}


pub struct CPU
{
    PC: u16, // program counter
    S: u8,   // stack pointer
    P: u8,   // processor status
    A: u8,   // accumulator
    X: u8,   // index register
    Y: u8    // index register
}

impl CPU
{
    pub fn new() -> CPU
    {
        CPU
        {
            PC: 0,
            S: 0,
            P: 0,
            A: 0,
            X: 0,
            Y: 0
        }        
    }
    
    pub fn reset(&self)
    {
        // set the registers to initial state on power up
    }

    pub fn update(&self)
    {
        //self.process_op(15);
        //self.process_op(16);
        // process opcodes, to the cpu stuff
    }     
 

    fn u8_to_enum(v: u8) -> opcodes::Opcodes
    {
        unsafe { mem::transmute(v) }
    }
    
    
    fn process_op(&self, opcode: u8)
    {
        use cpu::opcodes::Opcodes;
        match CPU::u8_to_enum(opcode)
        {
            Opcodes::BRK => println!("TODO: {}", opcode),
            Opcodes::NOP_zp   => (),
            Opcodes::NOP_abs  => (),
            Opcodes::HLT1 => panic!("Received HLT1 instruction: 0x{0:X}", opcode),            
            Opcodes::NOP_zpx  => (),
            Opcodes::NOP0     => (),
            Opcodes::NOP_abx  => (),
            Opcodes::HLT2 => panic!("Received HLT2 instruction: 0x{0:X}", opcode),
            Opcodes::HLT3 => panic!("Received HLT3 instruction: 0x{0:X}", opcode),         
            Opcodes::NOP_zpx2 => (),
            Opcodes::NOP1     => (),
            Opcodes::NOP_abx2 => (),
            Opcodes::HLT4 => panic!("Received HLT4 instruction: 0x{0:X}", opcode),            
            Opcodes::NOP2     => (),
            Opcodes::HLT5 => panic!("Received HLT5 instruction: 0x{0:X}", opcode),            
            Opcodes::NOP_zpx3 => (),
            Opcodes::NOP3     => (),
            Opcodes::NOP_abx3 => (),
            Opcodes::HLT6 => panic!("Received HLT6 instruction: 0x{0:X}", opcode),            
            Opcodes::NOP_zp2  => (),
            Opcodes::HLT7 => panic!("Received HLT7 instruction: 0x{0:X}", opcode),            
            Opcodes::NOP_zpx4 => (),
            Opcodes::NOP4     => (),
            Opcodes::NOP_abx4 => (),
            Opcodes::NOP_imm  => (),
            Opcodes::NOP_imm2 => (),
            Opcodes::NOP_imm3 => (),
            Opcodes::HLT8 => panic!("Received HLT8 instruction: 0x{0:X}", opcode),
            Opcodes::HLT9 => panic!("Received HLT9 instruction: 0x{0:X}", opcode),            
            Opcodes::NOP_imm4 => (),
            Opcodes::HLT10 => panic!("Received HLT10 instruction: 0x{0:X}", opcode),            
            Opcodes::NOP_zpx5 => (),
            Opcodes::NOP5     => (),
            Opcodes::NOP_abx5 => (),
            Opcodes::NOP_imm5 => (),           
            Opcodes::NOP      => (),
            Opcodes::HLT11 => panic!("Received HLT11 instruction: 0x{0:X}", opcode),            
            Opcodes::NOP_zpx6 => (),
            Opcodes::NOP6     => (),
            Opcodes::NOP_abx6 => (),
            _ => println!("Unknown opcode: 0x{0:X}", opcode)
        }        
    }
}
