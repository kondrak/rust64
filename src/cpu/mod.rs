// The CPU
#![allow(dead_code)]
#![allow(non_snake_case)]
mod opcodes;
use utils;
use memory;


// status flags for P register
enum StatusFlag
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

static RESET_VECTOR: u16 = 0xFFFC;
static IRQ_VECTOR:   u16 = 0xFFFE;

pub struct CPU
{
    PC: u16, // program counter
    SP: u8,  // stack pointer
    P: u8,   // processor status
    A: u8,   // accumulator
    X: u8,   // index register
    Y: u8,   // index register
    mem: memory::Memory // system memory (64k)
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

    fn set_status_flag(&mut self, flag: StatusFlag, value: bool)
    {
        if value
        {
            self.P |= flag as u8;
        }
        else
        {
            self.P &= !(flag as u8);
        }
    }

    fn get_status_flag(&mut self, flag: StatusFlag) -> bool
    {
        self.P & flag as u8 != 0x00
    }

    // these flags will be set in tandem quite often
    fn set_zn_flags(&mut self, value: u8)
    {
        self.set_status_flag(StatusFlag::Zero, value == 0x00);
        self.set_status_flag(StatusFlag::Negative, (value as i8) < 0);
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
        self.PC = self.mem.read_word_le(RESET_VECTOR);
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
    // TODO: some extra message if stack over/underflow occurs? (right now handled by Rust)
    fn push_byte(&mut self, value: u8)
    {
        self.SP -= 0x01;
        self.mem.write_byte(0x0100 + ((self.SP + 0x01) as u16) & 0x00FF, value);
    }

    fn pop_byte(&mut self) -> u8
    {
        let value = self.mem.read_byte(0x0100 + ((self.SP + 0x01) as u16) & 0x00FF);
        self.SP += 0x01;
        value
    }
    
    fn push_word(&mut self, value: u16)
    {
        self.SP -= 0x02;
        self.mem.write_word_le(0x0100 + ((self.SP + 0x01) as u16) & 0x00FF, value);
    }

    fn pop_word(&mut self) -> u16
    {
        let value = self.mem.read_word_le(0x0100 + ((self.SP + 0x01) as u16) & 0x00FF);
        self.PC += 0x02;
        value
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

    fn process_op(&mut self, opcode: u8) -> u8
    {
        match opcodes::get_instruction(opcode)
        {
            Some((instruction, num_cycles, addr_mode)) => {
                instruction.run(&addr_mode, self);
                num_cycles
            },
            None => panic!("No instruction - this should never happen! (0x{:02X} at ${:04X})", opcode, self.PC)
        }
    }
}
