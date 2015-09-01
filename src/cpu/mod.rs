// The CPU
#![allow(dead_code)]
#![allow(non_snake_case)]
mod opcodes;
use utils;
use std::mem;

struct Memory
{
    bytes: [u8;65536]
}

impl Memory
{
    pub fn new() -> Memory
    {
        Memory
        {
            bytes: [0;65536]
        }        
    }

    pub fn reset(&mut self)
    {
        self.write_byte_be(0x0000, 0xFF);
        self.write_byte_be(0x0001, 0x07);
    }

    // Write to memory using little endian memory address
    pub fn write_byte_le(&mut self, addr_le: u16, value: u8)
    {
        let addr_be = ((addr_le << 8) & 0xFF) | ((addr_le >> 8) & 0xFF);
        self.bytes[addr_be as usize] = value;
    }


    // Read from memory using little endian memory address
    pub fn read_byte_le(&mut self, addr_le: u16) -> u8
    {
        let addr_be = ((addr_le << 8) & 0xFF) | ((addr_le >> 8) & 0xFF);
        self.bytes[addr_be as usize]
    }
    

    // Write to memory using big endian memory address
    pub fn write_byte_be(&mut self, addr_be: u16, value: u8)
    {
        self.bytes[addr_be as usize] = value;
    }
    
    // Read from memory using big endian memory address
    pub fn read_byte_be(&mut self, addr_be: u16) -> u8
    {
        self.bytes[addr_be as usize]
    }

    // Read a word from memory, convert it to little endian
    pub fn read_word_lele(&mut self, addr_le: u16) -> u16
    {
        let addr_be = ((addr_le << 8) & 0xFF) | ((addr_le >> 8) & 0xFF);
        let value_be: u16 = ((self.bytes[addr_be as usize] as u16) << 8 & 0xFF00) |
                            ((self.bytes[(addr_be+1) as usize] as u16) & 0x00FF);

        let value_le: u16 = ((value_be << 8) & 0xFF00) | ((value_be >> 8) & 0x00FF);
        value_le
    }
     
    pub fn read_word_bele(&mut self, addr_be: u16) -> u16
    {
        let value_be: u16 = ((self.bytes[addr_be as usize] as u16) << 8 & 0xFF00) |
                            ((self.bytes[(addr_be+1) as usize] as u16) & 0x00FF);

        let value_le: u16 = ((value_be << 8) & 0xFF00) | ((value_be >> 8) & 0x00FF);
        value_le            
    }

    pub fn read_word_bebe(&mut self, addr_be: u16) -> u16
    {
        let value_be: u16 = ((self.bytes[addr_be as usize] as u16) << 8 & 0xFF00) |
                            ((self.bytes[(addr_be+1) as usize] as u16) & 0x00FF);
        value_be
    }
    
    pub fn read_word_lebe(&mut self, addr_le: u16) -> u16
    {
        let addr_be = ((addr_le << 8) & 0xFF) | ((addr_le >> 8) & 0xFF);
        let value_be: u16 = ((self.bytes[addr_be as usize] as u16) << 8 & 0xFF00) |
                            ((self.bytes[(addr_be+1) as usize] as u16) & 0x00FF);
        value_be     
    }

    pub fn write_word_bele(&mut self, addr_be: u16, value_be: u16)
    {
        let value_le_hi: u8 = (((value_be << 8) & 0xFF00) >> 8 & 0xFF) as u8;
        let value_le_lo: u8 = ((value_be >> 8) & 0x00FF) as u8;

        self.bytes[addr_be as usize] = value_le_hi;
        self.bytes[(addr_be + 0x01) as usize] = value_le_lo;
    }
    
}



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
    SP: u8,  // stack pointer
    P: u8,   // processor status
    A: u8,   // accumulator
    X: u8,   // index register
    Y: u8,   // index register
    mem: Memory
}

impl CPU
{
    pub fn new() -> CPU
    {
        CPU
        {
            PC: 0,
            SP: 0xFF,
            P: 0,
            A: 0,
            X: 0,
            Y: 0,
            mem: Memory::new()
        }        
    }
    
    pub fn reset(&mut self)
    {
        // set the registers to initial state on power up
        self.mem.reset();

        // load basic
        let mut startAddress: u32 = 0xA000;
        let basic = utils::open_file("rom/basic.rom");
        
        for (i,addr) in (startAddress..0xC000).enumerate()
        {
            self.mem.write_byte_be(addr as u16, basic[i as usize]);
        }

        // load chargen
        startAddress = 0xD000;
        let chargen = utils::open_file("rom/chargen.rom");
        
        for (i,addr) in (startAddress..0xE000).enumerate()
        {
            self.mem.write_byte_be(addr as u16, chargen[i as usize]);
        }
              
        // load kernal
        startAddress = 0xE000;
        let kernal = utils::open_file("rom/kernal.rom");
        
        for (i,addr) in (startAddress..0x10000).enumerate()
        {
            self.mem.write_byte_be(addr as u16, kernal[i as usize]);
        }

        // reset program counter
        self.PC = self.mem.read_word_bele(0xFFFC);     
    }

    pub fn update(&mut self)
    {
        let op = self.fetch_op();
        self.process_op(op);
        //self.process_op(15);
        //self.process_op(16);
        // process opcodes, to the cpu stuff
        //self.mem.bytes[0] = 1;

        //for i in (0..65536)
        //{
        //println!("{}", self.mem.bytes[i]);
        //}
    }     

    fn fetch_op(&mut self) -> u8
    {
        let op = self.mem.read_byte_be(self.PC);
        self.PC += 1;
        op
    }

    // stack memory: $0100 - $01FF (256 byes)
    fn push_byte(&mut self, value: u8)
    {
        self.SP -= 0x01;

        if self.SP == 0xFF
            { panic!("Stack underflow"); }
        
        self.mem.write_byte_be(0x0100 + ((self.SP + 0x01) as u16) & 0x00FF, value);
    }

    fn pop_byte(&mut self) -> u8
    {
        let value = self.mem.read_byte_be(0x0100 + ((self.SP + 0x01) as u16) & 0x00FF);
        self.PC += 0x01;

        if self.SP == 0x00
            { panic!("Stack overflow"); }
        
        value
    }
    
    fn push_word(&mut self, value: u16)
    {
        self.SP -= 0x02;

        if self.SP == 0xFF || self.SP == 0xFE
            { panic!("Stack underflow"); }
        
        self.mem.write_word_bele(0x0100 + ((self.SP + 0x01) as u16) & 0x00FF, value);
    }

    fn pop_word(&mut self) -> u16
    {
        let value = self.mem.read_word_bele(0x0100 + ((self.SP + 0x01) as u16) & 0x00FF);
        self.PC += 0x02;

        if self.SP == 0x00 || self.SP == 0x01
            { panic!("Stack overflow"); }
        
        value
    }

    fn u8_to_enum(v: u8) -> opcodes::Opcodes
    {
        unsafe { mem::transmute(v) }
    }

    fn mem_dump(&mut self)
    {
        for i in (0..0x10000)
        {
            let val = self.mem.read_byte_be(i as u16);
            if val != 0
                { println!("Addr: ${:04X} -> 0x{:02X}", i, val); }
        }        
    }
    
    
    fn process_op(&self, opcode: u8)
    {
        use cpu::opcodes::Opcodes;
        match CPU::u8_to_enum(opcode)
        {
            Opcodes::BRK => println!("TODO: {}", opcode),
            Opcodes::NOP_zp   => (),
            Opcodes::NOP_abs  => (),
            Opcodes::HLT1 => panic!("Received HLT1 instruction: 0x{:02X} at ${:02X}", opcode, self.PC),
            Opcodes::NOP_zpx  => (),
            Opcodes::NOP0     => (),
            Opcodes::NOP_abx  => (),
            Opcodes::HLT2 => panic!("Received HLT2 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            Opcodes::HLT3 => panic!("Received HLT3 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            Opcodes::NOP_zpx2 => (),
            Opcodes::NOP1     => (),
            Opcodes::NOP_abx2 => (),
            Opcodes::HLT4 => panic!("Received HLT4 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            Opcodes::NOP2     => (),
            Opcodes::HLT5 => panic!("Received HLT5 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            Opcodes::NOP_zpx3 => (),
            Opcodes::NOP3     => (),
            Opcodes::NOP_abx3 => (),
            Opcodes::HLT6 => panic!("Received HLT6 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            Opcodes::NOP_zp2  => (),
            Opcodes::HLT7 => panic!("Received HLT7 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            Opcodes::NOP_zpx4 => (),
            Opcodes::NOP4     => (),
            Opcodes::NOP_abx4 => (),
            Opcodes::NOP_imm  => (),
            Opcodes::NOP_imm2 => (),
            Opcodes::NOP_imm3 => (),
            Opcodes::HLT8 => panic!("Received HLT8 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            Opcodes::HLT9 => panic!("Received HLT9 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            Opcodes::NOP_imm4 => (),
            Opcodes::HLT10 => panic!("Received HLT10 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            Opcodes::NOP_zpx5 => (),
            Opcodes::NOP5     => (),
            Opcodes::NOP_abx5 => (),
            Opcodes::NOP_imm5 => (),           
            Opcodes::NOP      => (),
            Opcodes::HLT11 => panic!("Received HLT11 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            Opcodes::NOP_zpx6 => (),
            Opcodes::NOP6     => (),
            Opcodes::NOP_abx6 => (),
            _ => println!("Unknown opcode: 0x{:02X} at ${:04X}", opcode, self.PC)
        }        
    }
}
