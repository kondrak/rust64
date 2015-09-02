// The CPU
#![allow(dead_code)]
#![allow(non_snake_case)]
mod opcodes;
use utils;
use memory;
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
    SP: u8,  // stack pointer
    P: u8,   // processor status
    A: u8,   // accumulator
    X: u8,   // index register
    Y: u8,   // index register
    mem: memory::Memory
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
            mem: memory::Memory::new()
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
            self.mem.write_byte(addr as u16, basic[i as usize]);
        }

        // load chargen
        startAddress = 0xD000;
        let chargen = utils::open_file("rom/chargen.rom");
        
        for (i,addr) in (startAddress..0xE000).enumerate()
        {
            self.mem.write_byte(addr as u16, chargen[i as usize]);
        }
              
        // load kernal
        startAddress = 0xE000;
        let kernal = utils::open_file("rom/kernal.rom");
        
        for (i,addr) in (startAddress..0x10000).enumerate()
        {
            self.mem.write_byte(addr as u16, kernal[i as usize]);
        }

        // reset program counter
        self.PC = self.mem.read_word_le(0xFFFC);     
        self.SP = 0xFF;
	}

    pub fn update(&mut self)
    {
        let op = self.next_byte();
        self.process_op(op);
        //self.process_op(15);
        //self.process_op(16);
        // process opcodes, to the cpu stuff
        //self.mem.bytes[0] = 1;
    }     

    fn next_byte(&mut self) -> u8
    {
        let op = self.mem.read_byte(self.PC);
        self.PC += 1;
        op
    }

    fn next_word(&mut self) -> u16
    {
        let word = self.mem.read_word_le(self.PC);
        self.PC += 2;
        word
    }
    

    // stack memory: $0100 - $01FF (256 byes)
    fn push_byte(&mut self, value: u8)
    {
        self.SP -= 0x01;

        if self.SP == 0xFF
            { panic!("Stack underflow"); }
        
        self.mem.write_byte(0x0100 + ((self.SP + 0x01) as u16) & 0x00FF, value);
    }

    fn pop_byte(&mut self) -> u8
    {
        let value = self.mem.read_byte(0x0100 + ((self.SP + 0x01) as u16) & 0x00FF);
        self.SP += 0x01;

        if self.SP == 0x00
            { panic!("Stack overflow"); }
        
        value
    }
    
    fn push_word(&mut self, value: u16)
    {
        self.SP -= 0x02;

        if self.SP == 0xFF || self.SP == 0xFE
            { panic!("Stack underflow"); }
        
        self.mem.write_word_le(0x0100 + ((self.SP + 0x01) as u16) & 0x00FF, value);
    }

    fn pop_word(&mut self) -> u16
    {
        let value = self.mem.read_word_le(0x0100 + ((self.SP + 0x01) as u16) & 0x00FF);
        self.PC += 0x02;

        if self.SP == 0x00 || self.SP == 0x01
            { panic!("Stack overflow"); }
        
        value
    }

    // operand fetching - different addressing modes
    // implied addressing
    fn get_operand_impl(&mut self)
    {
        // do nothing?
    }

    // accumulator addressing
    fn get_operand_acc(&mut self) -> u8
    {
        self.A
    }

    // immediate addressing (operand stored at next byte)
    fn get_operand_imm(&mut self) -> u8
    {
        self.next_byte()
    }

    // absolute addressing (addr of operand stored in next word)
    fn get_operand_abs(&mut self) -> u8
    {
        let addr = self.next_word();
        self.mem.read_byte(addr)
    }

    // indexed absolute addressing (with X)
    fn get_operand_idx_absX(&mut self) -> u8
    {
        let nw = self.next_word();
        let addr = self.mem.read_word_le(nw);
        self.mem.read_byte(addr + self.X as u16)
    }

    // indexed absolute addressing (with Y)
    fn get_operand_idx_absY(&mut self) -> u8
    {
        let nw = self.next_word();
        let addr = self.mem.read_word_le(nw);
        self.mem.read_byte(addr + self.Y as u16)
    }    

    // zeropage addressing
    fn get_operand_zp(&mut self) -> u8
    {
        let nb = self.next_byte();
        self.mem.read_byte(nb as u16)
    }

    // indexed zeropage addressing (with X)
    fn get_operand_idx_zpX(&mut self) -> u8
    {
        let nb = self.next_byte();
        let addr = self.mem.read_word_le(nb as u16);
        self.mem.read_byte(addr + self.X as u16)
    }

    // indexed zeropage addressing (with Y)
    fn get_operand_idx_zpY(&mut self) -> u8
    {
        let nb = self.next_byte();
        let addr = self.mem.read_word_le(nb as u16);
        self.mem.read_byte(addr + self.Y as u16)
    }    

    // relative addressing
    fn get_operand_rel(&mut self) -> u8
    {
        let offset: i8 = self.next_byte() as i8;
        let addr: i16 = self.PC as i16 + offset as i16;
        self.mem.read_byte(addr as u16)
    }

    // absolute-indirect addressing
    fn get_operand_abs_ind(&mut self) -> u8
    {
        // same as abs?
        panic!("get_operand_abs_ind() not implemented")
    }

    // indexed-indirect addressing
    fn get_operand_idx_ind(&mut self) -> u8
    {
        let nb = self.next_byte();
        let addr = self.mem.read_word_le(nb as u16);
        self.mem.read_byte(addr + self.X as u16)
    }

    // indirect-indexed addressing
    fn get_operand_ind_idx(&mut self) -> u8
    {
        let nb = self.next_byte();
        let addr = self.mem.read_word_le(nb as u16);
        let finalAddr = self.mem.read_word_le(addr) + self.Y as u16;
        self.mem.read_byte(finalAddr)
    }

    // operand setting
    fn set_operand_acc(&mut self, value: u8)
    {
        self.A = value;
    }

    // immediate addressing (operand stored at next byte)
    fn set_operand_imm(&mut self, value: u8)
    {
        let addr = self.next_byte() as u16;
        self.mem.write_byte(addr, value);
    }

    // absolute addressing (addr of operand stored in next word)
    fn set_operand_abs(&mut self, value: u8)
    {
        let addr = self.next_word();
        self.mem.write_byte(addr, value);
    }

    // indexed absolute addressing (with X)
    fn set_operand_idx_absX(&mut self, value: u8)
    {
        let nw = self.next_word();
        let addr = self.mem.read_word_le(nw);
        self.mem.write_byte(addr + self.X as u16, value);
    }

    // indexed absolute addressing (with Y)
    fn set_operand_idx_absY(&mut self, value: u8)
    {
        let nw = self.next_word();
        let addr = self.mem.read_word_le(nw);
        self.mem.write_byte(addr + self.Y as u16, value);
    }    

    // zeropage addressing
    fn set_operand_zp(&mut self, value: u8)
    {
        let nb = self.next_byte() as u16;
        self.mem.write_byte(nb, value);
    }

    // indexed zeropage addressing (with X)
    fn set_operand_idx_zpX(&mut self, value: u8)
    {
        let nb = self.next_byte();
        let addr = self.mem.read_word_le(nb as u16);
        self.mem.write_byte(addr + self.X as u16, value);
    }

    // indexed zeropage addressing (with Y)
    fn set_operand_idx_zpY(&mut self, value: u8)
    {
        let nb = self.next_byte();
        let addr = self.mem.read_word_le(nb as u16);
        self.mem.write_byte(addr + self.Y as u16, value);
    }    

    // relative addressing
    fn set_operand_rel(&mut self, value: u8)
    {
        let offset: i8 = self.next_byte() as i8;
        let addr: i16 = self.PC as i16 + offset as i16;
        self.mem.write_byte(addr as u16, value);
    }

    // absolute-indirect addressing
    fn set_operand_abs_ind(&mut self, value: u8)
    {
        // same as abs?
        panic!("get_operand_abs_ind() not implemented")
    }

    // indexed-indirect addressing
    fn set_operand_idx_ind(&mut self, value: u8)
    {
        let nb = self.next_byte();
        let addr = self.mem.read_word_le(nb as u16);
        self.mem.write_byte(addr + self.X as u16, value);
    }

    // indirect-indexed addressing
    fn set_operand_ind_idx(&mut self, value: u8)
    {
        let nb = self.next_byte();
        let addr = self.mem.read_word_le(nb as u16);
        let finalAddr = self.mem.read_word_le(addr) + self.Y as u16;
        self.mem.write_byte(finalAddr, value);
    }
    
    
    
    fn u8_to_enum(v: u8) -> opcodes::Opcodes
    {
        unsafe { mem::transmute(v) }
    }

    fn mem_dump(&mut self)
    {
        for i in (0..0x10000)
        {
            let val = self.mem.read_byte(i as u16);
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
