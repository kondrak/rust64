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
    Negative         = 1 << 0,
    Overflow         = 1 << 1,
    Unused           = 1 << 2,
    Break            = 1 << 3,
    DecimalMode      = 1 << 4,
    InterruptDisable = 1 << 5,
    Zero             = 1 << 6,
    Carry            = 1 << 7 
}


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
            //Opcodes::BRK     => println!("TODO: {}", opcode),
            //Opcodes::ORA_izx => println!("TODO: {}", opcode),
            //Opcodes::HLT0    => panic!("Received HLT0 instruction: 0x(:02X} at ${:02X}", opcode, self.PC),
            //Opcodes::SLO_izx => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_zp  => (),
            //Opcodes::ORA_zp   => println!("TODO: {}", opcode),
            //Opcodes::ASL_zp   => println!("TODO: {}", opcode),
            //Opcodes::SLO_zp   => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::PHP      => println!("TODO: {}", opcode),
            //Opcodes::ORA_imm  => println!("TODO: {}", opcode),
            //Opcodes::ASL      => println!("TODO: {}", opcode),
            //Opcodes::ANC_imm  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_abs  => (),
            //Opcodes::ORA_abs  => println!("TODO: {}", opcode),
            //Opcodes::ASL_abs  => println!("TODO: {}", opcode),
            //Opcodes::SLO_abs  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::BPL_rel  => println!("TODO: {}", opcode),
            //Opcodes::ORA_izy  => println!("TODO: {}", opcode),
            Opcodes::HLT1     => panic!("Received HLT1 instruction: 0x{:02X} at ${:02X}", opcode, self.PC),
            //Opcodes::SLO_izy  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_zpx  => (),
            //Opcodes::ORA_zpx  => println!("TODO: {}", opcode),
            //Opcodes::ASL_zpx  => println!("TODO: {}", opcode),
            //Opcodes::SLO_zpx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::CLC      => println!("TODO: {}", opcode),
            //Opcodes::ORA_aby  => println!("TODO: {}", opcode),
            Opcodes::NOP0     => (),
            //Opcodes::SLO_aby  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_abx  => (),
            //Opcodes::ORA_abx  => println!("TODO: {}", opcode),
            //Opcodes::ASL_abx  => println!("TODO: {}", opcode),
            //Opcodes::SLO_abx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::JSR_abs  => println!("TODO: {}", opcode),
            //Opcodes::AND_izx  => println!("TODO: {}", opcode),
            Opcodes::HLT2     => panic!("Received HLT2 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            //Opcodes::RLA_izx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::BIT_zp   => println!("TODO: {}", opcode),
            //Opcodes::AND_zp   => println!("TODO: {}", opcode),
            //Opcodes::ROL_zp   => println!("TODO: {}", opcode),
            //Opcodes::RLA_zp   => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::PLP      => println!("TODO: {}", opcode),
            //Opcodes::AND_imm  => println!("TODO: {}", opcode),
            //Opcodes::ROL      => println!("TODO: {}", opcode),
            //Opcodes::ANC_im2  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::BIT_abs  => println!("TODO: {}", opcode),
            //Opcodes::AND_abs  => println!("TODO: {}", opcode),
            //Opcodes::ROL_abs  => println!("TODO: {}", opcode),
            //Opcodes::RLA_abs  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::BMI_rel  => println!("TODO: {}", opcode),
            //Opcodes::AND_izy  => println!("TODO: {}", opcode),
            Opcodes::HLT3     => panic!("Received HLT3 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            //Opcodes::RLA_izy  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_zpx2 => (),
            //Opcodes::AND_zpx  => println!("TODO: {}", opcode),
            //Opcodes::ROL_zpx  => println!("TODO: {}", opcode),
            //Opcodes::RLA_zpx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::SEC      => println!("TODO: {}", opcode),
            //Opcodes::AND_aby  => println!("TODO: {}", opcode),
            Opcodes::NOP1     => (),
            //Opcodes::RLA_aby  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_abx2 => (),
            //Opcodes::AND_abx  => println!("TODO: {}", opcode),
            //Opcodes::ROL_abx  => println!("TODO: {}", opcode),
            //Opcodes::RLA_abx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::RTI      => println!("TODO: {}", opcode),
            //Opcodes::EOR_izx  => println!("TODO: {}", opcode),
            Opcodes::HLT4     => panic!("Received HLT4 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            //Opcodes::SRE_izx  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP2     => (),
            //Opcodes::EOR_zp   => println!("TODO: {}", opcode),
            //Opcodes::LSR_zp   => println!("TODO: {}", opcode),
            //Opcodes::SRE_zp   => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::PHA      => println!("TODO: {}", opcode),
            //Opcodes::EOR_imm  => println!("TODO: {}", opcode),
            //Opcodes::LSR      => println!("TODO: {}", opcode),
            //Opcodes::ALR_imm  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::JMP_abs  => println!("TODO: {}", opcode),
            //Opcodes::EOR_abs  => println!("TODO: {}", opcode),
            //Opcodes::LSR_abs  => println!("TODO: {}", opcode),
            //Opcodes::SRE_abs  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::BVC_rel  => println!("TODO: {}", opcode),
            //Opcodes::EOR_izy  => println!("TODO: {}", opcode),
            Opcodes::HLT5     => panic!("Received HLT5 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            //Opcodes::SRE_izy  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_zpx3 => (),
            //Opcodes::EOR_zpx  => println!("TODO: {}", opcode),
            //Opcodes::LSR_zpx  => println!("TODO: {}", opcode),
            //Opcodes::SRE_zpx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::CLI      => println!("TODO: {}", opcode),
            //Opcodes::EOR_aby  => println!("TODO: {}", opcode),
            Opcodes::NOP3     => (),
            //Opcodes::SRE_aby  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_abx3 => (),
            //Opcodes::EOR_abx  => println!("TODO: {}", opcode),
            //Opcodes::LSR_abx  => println!("TODO: {}", opcode),
            //Opcodes::SRE_abx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::RTS      => println!("TODO: {}", opcode),
            //Opcodes::ADC_izx  => println!("TODO: {}", opcode),
            Opcodes::HLT6     => panic!("Received HLT6 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            //Opcodes::RRA_izx  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_zp2  => (),
            //Opcodes::ADC_zp   => println!("TODO: {}", opcode),
            //Opcodes::ROR_zp   => println!("TODO: {}", opcode),
            //Opcodes::RRA_zp   => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::PLA      => println!("TODO: {}", opcode),
            //Opcodes::ADC_imm  => println!("TODO: {}", opcode),
            //Opcodes::ROR      => println!("TODO: {}", opcode),
            //Opcodes::ARR      => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::JMP_ind  => println!("TODO: {}", opcode),
            //Opcodes::ADC_abs  => println!("TODO: {}", opcode),
            //Opcodes::ROR_abs  => println!("TODO: {}", opcode),
            //Opcodes::RRA_abs  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::BVS_rel  => println!("TODO: {}", opcode),
            //Opcodes::ADC_izy  => println!("TODO: {}", opcode),
            Opcodes::HLT7     => panic!("Received HLT7 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            //Opcodes::RRA_izy  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_zpx4 => (),
            //Opcodes::ADC_zpx  => println!("TODO: {}", opcode),
            //Opcodes::ROR_zpx  => println!("TODO: {}", opcode),
            //Opcodes::RRA_zpx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::SEI      => println!("TODO: {}", opcode),
            //Opcodes::ADC_aby  => println!("TODO: {}", opcode),
            Opcodes::NOP4     => (),
            //Opcodes::RRA_aby  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_abx4 => (),
            //Opcodes::ADC_abx  => println!("TODO: {}", opcode),
            //Opcodes::ROR_abx  => println!("TODO: {}", opcode),
            //Opcodes::RRA_abx  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_imm  => (),
            //Opcodes::STA_izx  => println!("TODO: {}", opcode),
            Opcodes::NOP_imm2 => (),
            //Opcodes::SAX_izx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::STY_zp   => println!("TODO: {}", opcode),
            //Opcodes::STA_zp   => println!("TODO: {}", opcode),
            //Opcodes::STX_zp   => println!("TODO: {}", opcode),
            //Opcodes::SAX_zp   => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::DEY      => println!("TODO: {}", opcode),
            Opcodes::NOP_imm3 => (),
            //Opcodes::TXA      => println!("TODO: {}", opcode),
            //Opcodes::XAA_imm  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::STY_abs  => println!("TODO: {}", opcode),
            //Opcodes::STA_abs  => println!("TODO: {}", opcode),
            //Opcodes::STX_abs  => println!("TODO: {}", opcode),
            //Opcodes::SAX_abs  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::BCC_rel  => println!("TODO: {}", opcode),
            //Opcodes::STA_izy  => println!("TODO: {}", opcode),
            Opcodes::HLT8     => panic!("Received HLT8 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            //Opcodes::AHX_izy  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::STY_zpx  => println!("TODO: {}", opcode),
            //Opcodes::STA_zpx  => println!("TODO: {}", opcode),
            //Opcodes::STX_zpy  => println!("TODO: {}", opcode),
            //Opcodes::SAX_zpy  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::TYA      => println!("TODO: {}", opcode),
            //Opcodes::STA_aby  => println!("TODO: {}", opcode),
            //Opcodes::TXS      => println!("TODO: {}", opcode),
            //Opcodes::TAS_aby  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::SHY_abx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::STA_abx  => println!("TODO: {}", opcode),
            //Opcodes::SHX_aby  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::AHX_aby  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::LDY_imm  => println!("TODO: {}", opcode),
            //Opcodes::LDA_izx  => println!("TODO: {}", opcode),
            //Opcodes::LDX_imm  => println!("TODO: {}", opcode),
            //Opcodes::LAX_izx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::LDY_zp   => println!("TODO: {}", opcode),
            //Opcodes::LDA_zp   => println!("TODO: {}", opcode),
            //Opcodes::LDX_zp   => println!("TODO: {}", opcode),
            //Opcodes::LAX_zp   => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::TAY      => println!("TODO: {}", opcode),
            //Opcodes::LDA_imm  => println!("TODO: {}", opcode),
            //Opcodes::TAX      => println!("TODO: {}", opcode),
            //Opcodes::LAX_imm  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::LDY_abs  => println!("TODO: {}", opcode),
            //Opcodes::LDA_abs  => println!("TODO: {}", opcode),
            //Opcodes::LDX_abs  => println!("TODO: {}", opcode),
            //Opcodes::LAX_abs  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::BCS_rel  => println!("TODO: {}", opcode),
            //Opcodes::LDA_izy  => println!("TODO: {}", opcode),
            Opcodes::HLT9     => panic!("Received HLT9 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            //Opcodes::LAX_izy  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::LDY_zpx  => println!("TODO: {}", opcode),
            //Opcodes::LDA_zpx  => println!("TODO: {}", opcode),
            //Opcodes::LDX_zpy  => println!("TODO: {}", opcode),
            //Opcodes::LAX_zpy  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::CLV      => println!("TODO: {}", opcode),
            //Opcodes::LDA_aby  => println!("TODO: {}", opcode),
            //Opcodes::TSX      => println!("TODO: {}", opcode),
            //Opcodes::LAS_aby  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::LDY_abx  => println!("TODO: {}", opcode),
            //Opcodes::LDA_abx  => println!("TODO: {}", opcode),
            //Opcodes::LDX_aby  => println!("TODO: {}", opcode),
            //Opcodes::LAX_aby  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::CPY_imm  => println!("TODO: {}", opcode),
            //Opcodes::CMP_izx  => println!("TODO: {}", opcode),
            Opcodes::NOP_imm4 => (),
            //Opcodes::DCP_izx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::CPY_zp   => println!("TODO: {}", opcode),
            //Opcodes::CMP_zp   => println!("TODO: {}", opcode),
            //Opcodes::DEC_zp   => println!("TODO: {}", opcode),
            //Opcodes::DCP_zp   => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::INY      => println!("TODO: {}", opcode),
            //Opcodes::CMP_imm  => println!("TODO: {}", opcode),
            //Opcodes::DEX      => println!("TODO: {}", opcode),
            //Opcodes::AXS_imm  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::CPY_abs  => println!("TODO: {}", opcode),
            //Opcodes::CMP_abs  => println!("TODO: {}", opcode),
            //Opcodes::DEC_abs  => println!("TODO: {}", opcode),
            //Opcodes::DCP_abs  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::BNE_rel  => println!("TODO: {}", opcode),
            //Opcodes::CMP_izy  => println!("TODO: {}", opcode),
            Opcodes::HLT10    => panic!("Received HLT10 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            //Opcodes::DCP_izy  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_zpx5 => (),
            //Opcodes::CMP_zpx  => println!("TODO: {}", opcode),
            //Opcodes::DEC_zpx  => println!("TODO: {}", opcode),
            //Opcodes::DCP_zpx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::CLD      => println!("TODO: {}", opcode),
            //Opcodes::CMP_aby  => println!("TODO: {}", opcode),
            Opcodes::NOP5     => (),
            //Opcodes::DCP_aby  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_abx5 => (),
            //Opcodes::CMP_abx  => println!("TODO: {}", opcode),
            //Opcodes::DEC_abx  => println!("TODO: {}", opcode),
            //Opcodes::DCP_abx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::CPX_imm  => println!("TODO: {}", opcode),
            //Opcodes::SBC_izx  => println!("TODO: {}", opcode),
            Opcodes::NOP_imm5 => (),
            //Opcodes::ISC_izx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::CPX_zp   => println!("TODO: {}", opcode),
            //Opcodes::SBC_zp   => println!("TODO: {}", opcode),
            //Opcodes::INC_zp   => println!("TODO: {}", opcode),
            //Opcodes::ISC_zp   => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::INX      => println!("TODO: {}", opcode),
            //Opcodes::SBC_imm  => println!("TODO: {}", opcode),
            Opcodes::NOP      => (),
            //Opcodes::SBC_imm2 => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::CPX      => println!("TODO: {}", opcode),
            //Opcodes::SBC_abs  => println!("TODO: {}", opcode),
            //Opcodes::INC_abs  => println!("TODO: {}", opcode),
            //Opcodes::ISC_abs  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::BEQ_rel  => println!("TODO: {}", opcode),
            //Opcodes::SBC_izy  => println!("TODO: {}", opcode),
            Opcodes::HLT11    => panic!("Received HLT11 instruction: 0x{:02X} at ${:04X}", opcode, self.PC),
            //Opcodes::ISC_izy  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_zpx6 => (),
            //Opcodes::SBC_zpx  => println!("TODO: {}", opcode),
            //Opcodes::INC_zpx  => println!("TODO: {}", opcode),
            //Opcodes::ISC_zpx  => println!("TODO: (FORBIDDEN) {}", opcode),
            //Opcodes::SED      => println!("TODO: {}", opcode),
            //Opcodes::SBC_aby  => println!("TODO: {}", opcode),
            Opcodes::NOP6     => (),
            //Opcodes::ISC_aby  => println!("TODO: (FORBIDDEN) {}", opcode),
            Opcodes::NOP_abx6 => (),
            //Opcodes::SBC_abx  => println!("TODO: {}", opcode),
            //Opcodes::INC_abx  => println!("TODO: {}", opcode),
            //Opcodes::ISC_abx  => println!("TODO: (FORBIDDEN) {}", opcode),
            _                 => println!("Unknown opcode: 0x{:02X} at ${:04X}", opcode, self.PC)
        }
    }
}
