use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::path::Path;

use c64::cpu;
use c64::opcodes;

// helper macros to easily extract references from Option<RefCell<...>>
macro_rules! as_ref {
    ($x:expr) => ($x.as_ref().unwrap().borrow_mut())
}

macro_rules! as_mut {
    ($x:expr) => ($x.as_mut().unwrap().borrow_mut())
}

pub fn open_file(filename: &str, offset: u64) -> Vec<u8>
{
    let path = Path::new(&filename);
    
    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", path.display(), Error::description(&why)),
        Ok(file) => file,
    };

    let mut file_data = Vec::<u8>::new();

    let _ = file.seek(SeekFrom::Start(offset));
    let result = file.read_to_end(&mut file_data);
    
    match result {
        Err(why)   => panic!("Error reading file: {}", Error::description(&why)),
        Ok(result) => println!("Read {}: {} bytes", path.display(), result),
    };    

    file_data
}

// instruction debugging
pub struct OpDebugger
{
    pub jump_queue: Vec<u8>
}

impl OpDebugger
{
    pub fn new() -> OpDebugger
    {
        OpDebugger
        {
            jump_queue: Vec::<u8>::new()
        }
    }
}

pub fn debug_instruction(opcode: u8, instruction: Option<(&opcodes::Op, u8, &opcodes::AddrMode)>, cpu: &mut cpu::CPU)
{
    match instruction
    {
        Some((instruction, num_cycles, addr_mode)) => {
            let mut operand_hex: String;
            let mut operand: String;

            // RTS? pop from queue to continue logging
            match *instruction
            {
                opcodes::Op::RTS => { let _ = cpu.op_debugger.jump_queue.pop(); return; },
                opcodes::Op::JSR => if !cpu.op_debugger.jump_queue.is_empty() { cpu.op_debugger.jump_queue.push(opcode); return; },
                _ => if !cpu.op_debugger.jump_queue.is_empty() { return; }
            }

            match *addr_mode {
                opcodes::AddrMode::Implied => {
                    operand_hex = format!("       ");
                    operand = format!("       ");
                },
                opcodes::AddrMode::Accumulator => {
                    operand_hex = format!("       ");
                    operand = format!("A      ");
                },
                opcodes::AddrMode::Immediate(..) => {
                    operand_hex = format!(" {:02X}    ", cpu.read_byte(cpu.prev_PC));
                    operand = format!("#${:02X}   ", cpu.read_byte(cpu.prev_PC)); 
                },
                opcodes::AddrMode::Absolute(..) => {
                    operand_hex = format!(" {:02X} {:02X} ", cpu.read_byte(cpu.prev_PC), cpu.read_byte(cpu.prev_PC + 0x01));
                    operand = format!("${:04X}  ", cpu.read_word_le(cpu.prev_PC));
                },
                opcodes::AddrMode::AbsoluteIndexedX(..) => {
                    operand_hex = format!(" {:02X} {:02X} ", cpu.read_byte(cpu.prev_PC), cpu.read_byte(cpu.prev_PC + 0x01));
                    operand = format!("${:04X},X", cpu.read_word_le(cpu.prev_PC));
                },
                opcodes::AddrMode::AbsoluteIndexedY(..) => {
                    operand_hex = format!(" {:02X} {:02X} ", cpu.read_byte(cpu.prev_PC), cpu.read_byte(cpu.prev_PC + 0x01));
                    operand = format!("${:04X},Y", cpu.read_word_le(cpu.prev_PC));
                },
                opcodes::AddrMode::Zeropage(..) => {
                    operand_hex = format!(" {:02X}    ", cpu.read_byte(cpu.prev_PC));
                    operand = format!("${:02X}    ", cpu.read_byte(cpu.prev_PC));
                }, 
                opcodes::AddrMode::ZeropageIndexedX(..) => {
                    operand_hex = format!(" {:02X}    ", cpu.read_byte(cpu.prev_PC));
                    operand = format!("${:02X},X", cpu.read_byte(cpu.prev_PC));
                },
                opcodes::AddrMode::ZeropageIndexedY(..) => {
                    operand_hex = format!(" {:02X}    ", cpu.read_byte(cpu.prev_PC));
                    operand = format!("${:02X},Y", cpu.read_byte(cpu.prev_PC));
                },
                opcodes::AddrMode::Relative(..) => {
                    operand_hex = format!(" {:02X}    ", cpu.read_byte(cpu.prev_PC));
                    let b: i8 = cpu.read_byte(cpu.prev_PC) as i8;
                    operand = format!("${:04X}  ", ((cpu.prev_PC + 1) as i16 + b as i16) as u16);
                },
                opcodes::AddrMode::Indirect(..) => {
                    operand_hex = format!(" {:02X} {:02X} ", cpu.read_byte(cpu.prev_PC), cpu.read_byte(cpu.prev_PC + 0x01));
                    operand = format!("(${:04X})", cpu.read_word_le(cpu.prev_PC));
                },
                opcodes::AddrMode::IndexedIndirectX(..) => {
                    operand_hex = format!(" {:02X}    ", cpu.read_byte(cpu.prev_PC));
                    operand = format!("(${:02X},X)", cpu.read_byte(cpu.prev_PC));
                },
                opcodes::AddrMode::IndirectIndexedY(..) => {
                    operand_hex = format!(" {:02X}    ", cpu.read_byte(cpu.prev_PC));
                    operand = format!("(${:02X}),Y", cpu.read_byte(cpu.prev_PC));
                },
            }

            println!("${:04X}: {:02X}{} {} {}    <- A: {:02X} X: {:02X} Y: {:02X} SP: {:02X} 00: {:02X} 01: {:02X} CZIDB-VN: [{:08b}] ({} cycles)", cpu.prev_PC - 1, opcode, operand_hex, instruction, operand, cpu.A, cpu.X, cpu.Y, cpu.SP, cpu.read_byte(0x0000), cpu.read_byte(0x0001), cpu.P, num_cycles);

            // JSR? push on queue to supress logging
            match *instruction
            {
                opcodes::Op::JSR => cpu.op_debugger.jump_queue.push(opcode),
                _ => ()
            }
        },
        None => ()
    }
}
