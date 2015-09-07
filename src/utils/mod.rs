use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

use cpu;

pub fn open_file(filename: &str) -> Vec<u8>
{
    let path = Path::new(&filename);
    
    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", path.display(), Error::description(&why)),
        Ok(file) => file,
    };
    
    let mut file_data = Vec::<u8>::new();

    let result = file.read_to_end(&mut file_data);

    match result {
        Err(why)   => panic!("Error reading file: {}", Error::description(&why)),
        Ok(result) => println!("Read {}: {} bytes", path.display(), result),
    };    

    file_data
}

pub fn debug_instruction(opcode: u8, cpu: &mut cpu::CPU)
{
    match cpu::opcodes::get_instruction(opcode)
    {
        Some((instruction, num_cycles, addr_mode)) => {
            // get the operand
            cpu.mem.read_byte((cpu.PC - 0x01) as u16);
            let mut operand_hex: String;
            let mut operand: String;
            
            match addr_mode {
                cpu::opcodes::AddrMode::Implied => {
                    operand_hex = format!("       ");
                    operand = format!("       ");
                },
                cpu::opcodes::AddrMode::Accumulator => {
                    operand_hex = format!("       ");
                    operand = format!("       ");
                },
                cpu::opcodes::AddrMode::Immediate => {
                    operand_hex = format!(" {:02X}    ", cpu.mem.read_byte(cpu.PC));
                    operand = format!("#${:02X}   ", cpu.mem.read_byte(cpu.PC)); 
                },
                cpu::opcodes::AddrMode::Absolute => {
                    operand_hex = format!(" {:02X} {:02X} ", cpu.mem.read_byte(cpu.PC), cpu.mem.read_byte(cpu.PC + 0x01));
                    operand = format!("${:04X}  ", cpu.mem.read_word_le(cpu.PC));
                },
                cpu::opcodes::AddrMode::AbsoluteIndexedX => {
                    operand_hex = format!(" {:02X} {:02X} ", cpu.mem.read_byte(cpu.PC), cpu.mem.read_byte(cpu.PC + 0x01));
                    operand = format!("${:04X},X", cpu.mem.read_word_le(cpu.PC));
                },
                cpu::opcodes::AddrMode::AbsoluteIndexedY => {
                    operand_hex = format!(" {:02X} {:02X} ", cpu.mem.read_byte(cpu.PC), cpu.mem.read_byte(cpu.PC + 0x01));
                    operand = format!("${:04X},Y", cpu.mem.read_word_le(cpu.PC));
                },
                cpu::opcodes::AddrMode::Zeropage => {
                    operand_hex = format!(" {:02X}    ", cpu.mem.read_byte(cpu.PC));
                    operand = format!("${:02X}    ", cpu.mem.read_byte(cpu.PC));
                }, 
                cpu::opcodes::AddrMode::ZeropageIndexedX => {
                    operand_hex = format!(" {:02X}    ", cpu.mem.read_byte(cpu.PC));
                    operand = format!("${:02X},X", cpu.mem.read_byte(cpu.PC));
                },
                cpu::opcodes::AddrMode::ZeropageIndexedY => {
                    operand_hex = format!(" {:02X}    ", cpu.mem.read_byte(cpu.PC));
                    operand = format!("${:02X},Y", cpu.mem.read_byte(cpu.PC));
                },
                cpu::opcodes::AddrMode::Relative => {
                    operand_hex = format!(" {:02X}    ", cpu.mem.read_byte(cpu.PC));
                    operand = format!("${:04X}  ", (cpu.PC as i16 + cpu.mem.read_byte(cpu.PC) as i16) as u16);
                },
                cpu::opcodes::AddrMode::Indirect => {
                    operand_hex = format!(" {:02X} {:02X} ", cpu.mem.read_byte(cpu.PC), cpu.mem.read_byte(cpu.PC + 0x01));
                    operand = format!("(${:04X})", cpu.mem.read_word_le(cpu.PC));
                },
                cpu::opcodes::AddrMode::IndexedIndirectX => {
                    operand_hex = format!(" {:02X} {:02X} ", cpu.mem.read_byte(cpu.PC), cpu.mem.read_byte(cpu.PC + 0x01));
                    operand = format!("(${:02X},X)", cpu.mem.read_byte(cpu.PC));
                },
                cpu::opcodes::AddrMode::IndirectIndexedY => {
                    operand_hex = format!(" {:02X} {:02X} ", cpu.mem.read_byte(cpu.PC), cpu.mem.read_byte(cpu.PC + 0x01));
                    operand = format!("(${:02X}),Y", cpu.mem.read_byte(cpu.PC));
                },
                //_ => operand_hex = panic!("Unknown addressing mode?")
            }

            println!("${:04X}: {:02X}{} {} {}      <- A: {:02X} X: {:02X} Y: {:02X} SP: {:02X} CZIDB-VN: [{:08b}] ({} cycles)", cpu.PC - 0x01, opcode, operand_hex, instruction, operand, cpu.A, cpu.X, cpu.Y, cpu.SP, cpu.P, num_cycles);
        },
        None => ()
    }
}
