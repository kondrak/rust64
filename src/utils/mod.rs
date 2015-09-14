use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::path::Path;

use cpu;

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

pub fn debug_instruction(opcode: u8, instruction: Option<(&cpu::opcodes::Op, u8, &cpu::opcodes::AddrMode)>, cpu: &mut cpu::CPU, oldpc: u16)
{
    match instruction
    {
        Some((instruction, num_cycles, addr_mode)) => {
            let mut operand_hex: String;
            let mut operand: String;
            
            match *addr_mode {
                cpu::opcodes::AddrMode::Implied => {
                    operand_hex = format!("       ");
                    operand = format!("       ");
                },
                cpu::opcodes::AddrMode::Accumulator => {
                    operand_hex = format!("       ");
                    operand = format!("A      ");
                },
                cpu::opcodes::AddrMode::Immediate(..) => {
                    operand_hex = format!(" {:02X}    ", cpu.mem.read_byte(oldpc));
                    operand = format!("#${:02X}   ", cpu.mem.read_byte(oldpc)); 
                },
                cpu::opcodes::AddrMode::Absolute(..) => {
                    operand_hex = format!(" {:02X} {:02X} ", cpu.mem.read_byte(oldpc), cpu.mem.read_byte(oldpc + 0x01));
                    operand = format!("${:04X}  ", cpu.mem.read_word_le(oldpc));
                },
                cpu::opcodes::AddrMode::AbsoluteIndexedX(..) => {
                    operand_hex = format!(" {:02X} {:02X} ", cpu.mem.read_byte(oldpc), cpu.mem.read_byte(oldpc + 0x01));
                    operand = format!("${:04X},X", cpu.mem.read_word_le(oldpc));
                },
                cpu::opcodes::AddrMode::AbsoluteIndexedY(..) => {
                    operand_hex = format!(" {:02X} {:02X} ", cpu.mem.read_byte(oldpc), cpu.mem.read_byte(oldpc + 0x01));
                    operand = format!("${:04X},Y", cpu.mem.read_word_le(oldpc));
                },
                cpu::opcodes::AddrMode::Zeropage(..) => {
                    operand_hex = format!(" {:02X}    ", cpu.mem.read_byte(oldpc));
                    operand = format!("${:02X}    ", cpu.mem.read_byte(oldpc));
                }, 
                cpu::opcodes::AddrMode::ZeropageIndexedX(..) => {
                    operand_hex = format!(" {:02X}    ", cpu.mem.read_byte(oldpc));
                    operand = format!("${:02X},X", cpu.mem.read_byte(oldpc));
                },
                cpu::opcodes::AddrMode::ZeropageIndexedY(..) => {
                    operand_hex = format!(" {:02X}    ", cpu.mem.read_byte(oldpc));
                    operand = format!("${:02X},Y", cpu.mem.read_byte(oldpc));
                },
                cpu::opcodes::AddrMode::Relative(..) => {
                    operand_hex = format!(" {:02X}    ", cpu.mem.read_byte(oldpc));
                    let b: i8 = cpu.mem.read_byte(oldpc) as i8;
                    operand = format!("${:04X}  ", ((oldpc + 1) as i16 + b as i16) as u16);
                },
                cpu::opcodes::AddrMode::Indirect(..) => {
                    operand_hex = format!(" {:02X} {:02X} ", cpu.mem.read_byte(oldpc), cpu.mem.read_byte(oldpc + 0x01));
                    operand = format!("(${:04X})", cpu.mem.read_word_le(oldpc));
                },
                cpu::opcodes::AddrMode::IndexedIndirectX(..) => {
                    operand_hex = format!(" {:02X}    ", cpu.mem.read_byte(oldpc));
                    operand = format!("(${:02X},X)", cpu.mem.read_byte(oldpc));
                },
                cpu::opcodes::AddrMode::IndirectIndexedY(..) => {
                    operand_hex = format!(" {:02X}    ", cpu.mem.read_byte(oldpc));
                    operand = format!("(${:02X}),Y", cpu.mem.read_byte(oldpc));
                },
                //_ => operand_hex = panic!("Unknown addressing mode?")
            }

            println!("${:04X}: {:02X}{} {} {}    <- A: {:02X} X: {:02X} Y: {:02X} SP: {:02X} CZIDB-VN: [{:08b}] ({} cycles)", oldpc - 1, opcode, operand_hex, instruction, operand, cpu.A, cpu.X, cpu.Y, cpu.SP, cpu.P, num_cycles);
        },
        None => ()
    }
}
