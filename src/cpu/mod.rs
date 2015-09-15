// The CPU
#![allow(dead_code)]
#![allow(non_snake_case)]
pub mod opcodes;
extern crate sdl2;
use utils;
use memory;
use video;

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

static NMI_VECTOR:   u16 = 0xFFFA;
static RESET_VECTOR: u16 = 0xFFFC;
static IRQ_VECTOR:   u16 = 0xFFFE;

pub struct CPU
{
    pub PC: u16, // program counter
    pub SP: u8,  // stack pointer
    pub P: u8,   // processor status
    pub A: u8,   // accumulator
    pub X: u8,   // index register
    pub Y: u8,   // index register
    pub mem: memory::Memory, // system memory (64k)
    pub font: video::font::SysFont,
    pub op_debugger: utils::OpDebugger
}

impl CPU
{
    pub fn new(renderer: &sdl2::render::Renderer) -> CPU
    {
        CPU
        {
            PC: 0,
            SP: 0xFF,
            P: 0,
            A: 0,
            X: 0,
            Y: 0,
            mem: memory::Memory::new(),
            font: video::font::SysFont::new(renderer),
            op_debugger: utils::OpDebugger::new()
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

        // init subroutine with memory test
        //self.mem.debug_write_rom(0xFCF5, 0xEA);
        //self.mem.debug_write_rom(0xFCF6, 0xEA);
        //self.mem.debug_write_rom(0xFCF7, 0xEA);

        // the memory test (very slow right now)
        self.mem.debug_write_rom(0xFD86, 0xD0);
        // reset program counter
        self.PC = self.mem.read_word_le(RESET_VECTOR);
        self.SP = 0xFF;
    }

    pub fn update(&mut self)
    {
        let op = self.next_byte();
        self.process_op(op);
        self.process_nmi();
        self.process_irq();
    }

    pub fn render(&mut self, renderer: &mut sdl2::render::Renderer)
    {
        // dump screen memory
        let mut start = 0x0400;

        for y in 0..25
        {
            for x in 0..40
            {
                let d = self.mem.read_byte(start);
                self.font.draw_char(renderer, x, y, d);
                start += 1;
            }
        }
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
        self.mem.write_byte(0x0100 + (self.SP + 0x01) as u16, value);
    }

    fn pop_byte(&mut self) -> u8
    {
        let value = self.mem.read_byte(0x0100 + (self.SP + 0x01) as u16);
        self.SP += 0x01;
        value
    }

    fn push_word(&mut self, value: u16)
    {
        self.SP -= 0x02;
        self.mem.write_word_le(0x0100 + (self.SP + 0x01) as u16, value);
    }

    fn pop_word(&mut self) -> u16
    {
        let value = self.mem.read_word_le(0x0100 + (self.SP + 0x01) as u16);
        self.SP += 0x02;
        value
    }

    fn process_nmi(&mut self)
    {
        // TODO: non-maskable irq
    }
    
    fn process_irq(&mut self)
    {
        if !self.get_status_flag(StatusFlag::InterruptDisable)
        {
            // TODO
            println!("IRQ processing");
        }
    }
    
    fn process_op(&mut self, opcode: u8) -> u8
    {
        //utils::debug_instruction(opcode, self);
        let oldpc = self.PC;
        match opcodes::get_instruction(opcode, self)
        {
            Some((instruction, num_cycles, addr_mode)) => {
                utils::debug_instruction(opcode, Some((&instruction, num_cycles, &addr_mode)), self, oldpc);
                instruction.run(&addr_mode, self);
                //println!("Stack pop: {:04X}", self.mem.read_word_le(0x0100 + (0xFD + 0x01) as u16));
                num_cycles
            },
            None => panic!("No instruction - this should never happen! (0x{:02X} at ${:04X})", opcode, self.PC)
        }
    }
}
