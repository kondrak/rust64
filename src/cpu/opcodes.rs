// opcode enumeration suffix: // addressing mode:
// imm = #$00                 // immediate 
// zp = $00                   // zero page
// zpx = $00,X                // zero page with X
// zpy = $00,Y                // zero page with Y
// izx = ($00,X)              // indexed indirect (X)
// izy = ($00),Y              // indirect indexed (Y)
// abs = $0000                // absolute
// abx = $0000,X              // indexed absolute with X
// aby = $0000,Y              // indexed absolute with Y
// ind = ($0000)              // indirect
// rel = $0000                // relative to PC/IP

#![allow(dead_code)]
#![allow(non_camel_case_types)]

use cpu;

pub enum AddrMode
{
    Implied,
    Accumulator,
    Immediate,
    Absolute,
    IndexedAbsoluteX,
    IndexedAbsoluteY,
    Zeropage,
    ZeropageIndexedX,
    ZeropageIndexedY,
    Relative,
    AbsoluteIndirect,
    IndexedIndirectX,
    IndirectIndexedY
}

pub enum Opcodes
{
    BRK      = 0x00,
    ORA_izx  = 0x01,
    HLT0     = 0x02, // forbidden opcode
    SLO_izx  = 0x03, // forbidden opcode
    NOP_zp   = 0x04, // forbidden opcode
    ORA_zp   = 0x05,
    ASL_zp   = 0x06,
    SLO_zp   = 0x07, // forbidden opcode
    PHP      = 0x08,
    ORA_imm  = 0x09,
    ASL      = 0x0A,
    ANC_imm  = 0x0B, // forbidden opcode
    NOP_abs  = 0x0C, // forbidden opcode
    ORA_abs  = 0x0D,
    ASL_abs  = 0x0E,
    SLO_abs  = 0x0F, // forbidden opcode
    BPL_rel  = 0x10,
    ORA_izy  = 0x11,
    HLT1     = 0x12, // forbidden opcode
    SLO_izy  = 0x13, // forbidden opcode
    NOP_zpx  = 0x14, // forbidden opcode
    ORA_zpx  = 0x15,
    ASL_zpx  = 0x16,
    SLO_zpx  = 0x17, // forbidden opcode
    CLC      = 0x18,
    ORA_aby  = 0x19,
    NOP0     = 0x1A, // forbidden opcode
    SLO_aby  = 0x1B, // forbidden opcode
    NOP_abx  = 0x1C, // forbidden opcode
    ORA_abx  = 0x1D,
    ASL_abx  = 0x1E,
    SLO_abx  = 0x1F, // forbidden opcode
    JSR_abs  = 0x20,
    AND_izx  = 0x21,
    HLT2     = 0x22, // forbidden opcode
    RLA_izx  = 0x23, // forbidden opcode
    BIT_zp   = 0x24,
    AND_zp   = 0x25,
    ROL_zp   = 0x26,
    RLA_zp   = 0x27, // forbidden opcode
    PLP      = 0x28,
    AND_imm  = 0x29,
    ROL      = 0x2A,
    ANC_im2  = 0x2B, // forbidden opcode
    BIT_abs  = 0x2C,
    AND_abs  = 0x2D,
    ROL_abs  = 0x2E,
    RLA_abs  = 0x2F, // forbidden opcode
    BMI_rel  = 0x30,
    AND_izy  = 0x31,
    HLT3     = 0x32, // forbidden opcode
    RLA_izy  = 0x33, // forbidden opcode
    NOP_zpx2 = 0x34, // forbidden opcode
    AND_zpx  = 0x35,
    ROL_zpx  = 0x36,
    RLA_zpx  = 0x37, // forbidden opcode
    SEC      = 0x38,
    AND_aby  = 0x39,
    NOP1     = 0x3A, // forbidden opcode
    RLA_aby  = 0x3B, // forbidden opcode
    NOP_abx2 = 0x3C, // forbidden opcode
    AND_abx  = 0x3D,
    ROL_abx  = 0x3E,
    RLA_abx  = 0x3F, // forbidden opcode
    RTI      = 0x40,
    EOR_izx  = 0x41,
    HLT4     = 0x42, // forbidden opcode
    SRE_izx  = 0x43, // forbidden opcode
    NOP2     = 0x44, // forbidden opcode
    EOR_zp   = 0x45,
    LSR_zp   = 0x46,
    SRE_zp   = 0x47, // forbidden opcode
    PHA      = 0x48,
    EOR_imm  = 0x49,
    LSR      = 0x4A,
    ALR_imm  = 0x4B, // forbidden opcode
    JMP_abs  = 0x4C,
    EOR_abs  = 0x4D,
    LSR_abs  = 0x4E,
    SRE_abs  = 0x4F, // forbidden opcode
    BVC_rel  = 0x50,
    EOR_izy  = 0x51,
    HLT5     = 0x52, // forbidden opcode
    SRE_izy  = 0x53, // forbidden opcode
    NOP_zpx3 = 0x54, // forbidden opcode
    EOR_zpx  = 0x55,
    LSR_zpx  = 0x56,
    SRE_zpx  = 0x57, // forbidden opcode
    CLI      = 0x58,
    EOR_aby  = 0x59,
    NOP3     = 0x5A, // forbidden opcode
    SRE_aby  = 0x5B, // forbidden opcode
    NOP_abx3 = 0x5C, // forbidden opcode
    EOR_abx  = 0x5D,
    LSR_abx  = 0x5E,
    SRE_abx  = 0x5F, // forbidden opcode
    RTS      = 0x60,
    ADC_izx  = 0x61,
    HLT6     = 0x62, // forbidden opcode
    RRA_izx  = 0x63, // forbidden opcode
    NOP_zp2  = 0x64, // forbidden opcode
    ADC_zp   = 0x65,
    ROR_zp   = 0x66,
    RRA_zp   = 0x67, // forbidden opcode
    PLA      = 0x68,
    ADC_imm  = 0x69,
    ROR      = 0x6A,
    ARR      = 0x6B, // forbidden opcode
    JMP_ind  = 0x6C,
    ADC_abs  = 0x6D,
    ROR_abs  = 0x6E,
    RRA_abs  = 0x6F, // forbidden opcode
    BVS_rel  = 0x70,
    ADC_izy  = 0x71,
    HLT7     = 0x72, // forbidden opcode
    RRA_izy  = 0x73, // forbidden opcode
    NOP_zpx4 = 0x74, // forbidden opcode
    ADC_zpx  = 0x75,
    ROR_zpx  = 0x76,
    RRA_zpx  = 0x77, // forbidden opcode
    SEI      = 0x78,
    ADC_aby  = 0x79,
    NOP4     = 0x7A, // forbidden opcode
    RRA_aby  = 0x7B, // forbidden opcode
    NOP_abx4 = 0x7C, // forbidden opcode
    ADC_abx  = 0x7D,
    ROR_abx  = 0x7E,
    RRA_abx  = 0x7F, // forbidden opcode
    NOP_imm  = 0x80, // forbidden opcode
    STA_izx  = 0x81,
    NOP_imm2 = 0x82, // forbidden opcode
    SAX_izx  = 0x83, // forbidden opcode
    STY_zp   = 0x84,
    STA_zp   = 0x85,
    STX_zp   = 0x86,
    SAX_zp   = 0x87, // forbidden opcode
    DEY      = 0x88,
    NOP_imm3 = 0x89, // forbidden opcode
    TXA      = 0x8A,
    XAA_imm  = 0x8B, // forbidden opcode
    STY_abs  = 0x8C,
    STA_abs  = 0x8D,
    STX_abs  = 0x8E,
    SAX_abs  = 0x8F, // forbidden opcode
    BCC_rel  = 0x90,
    STA_izy  = 0x91,
    HLT8     = 0x92, // forbidden opcode
    AHX_izy  = 0x93, // forbidden opcode
    STY_zpx  = 0x94,
    STA_zpx  = 0x95,
    STX_zpy  = 0x96,
    SAX_zpy  = 0x97, // forbidden opcode
    TYA      = 0x98,
    STA_aby  = 0x99,
    TXS      = 0x9A,
    TAS_aby  = 0x9B, // forbidden opcode
    SHY_abx  = 0x9C, // forbidden opcode
    STA_abx  = 0x9D,
    SHX_aby  = 0x9E, // forbidden opcode
    AHX_aby  = 0x9F, // forbidden opcode
    LDY_imm  = 0xA0,
    LDA_izx  = 0xA1,
    LDX_imm  = 0xA2,
    LAX_izx  = 0xA3, // forbidden opcode
    LDY_zp   = 0xA4,
    LDA_zp   = 0xA5,
    LDX_zp   = 0xA6,
    LAX_zp   = 0xA7, // forbidden opcode
    TAY      = 0xA8,
    LDA_imm  = 0xA9,
    TAX      = 0xAA,
    LAX_imm  = 0xAB, // forbidden opcode
    LDY_abs  = 0xAC,
    LDA_abs  = 0xAD,
    LDX_abs  = 0xAE,
    LAX_abs  = 0xAF, // forbidden opcode
    BCS_rel  = 0xB0,
    LDA_izy  = 0xB1,
    HLT9     = 0xB2, // forbidden opcode
    LAX_izy  = 0xB3, // forbidden opcode
    LDY_zpx  = 0xB4,
    LDA_zpx  = 0xB5,
    LDX_zpy  = 0xB6,
    LAX_zpy  = 0xB7, // forbidden opcode
    CLV      = 0xB8,
    LDA_aby  = 0xB9,
    TSX      = 0xBA,
    LAS_aby  = 0xBB, // forbidden opcode
    LDY_abx  = 0xBC,
    LDA_abx  = 0xBD,
    LDX_aby  = 0xBE,
    LAX_aby  = 0xBF, // forbidden opcode
    CPY_imm  = 0xC0,
    CMP_izx  = 0xC1,
    NOP_imm4 = 0xC2, // forbidden opcode
    DCP_izx  = 0xC3, // forbidden opcode
    CPY_zp   = 0xC4,
    CMP_zp   = 0xC5,
    DEC_zp   = 0xC6,
    DCP_zp   = 0xC7, // forbidden opcode
    INY      = 0xC8,
    CMP_imm  = 0xC9,
    DEX      = 0xCA,
    AXS_imm  = 0xCB, // forbidden opcode
    CPY_abs  = 0xCC,
    CMP_abs  = 0xCD,
    DEC_abs  = 0xCE,
    DCP_abs  = 0xCF, // forbidden opcode
    BNE_rel  = 0xD0,
    CMP_izy  = 0xD1,
    HLT10    = 0xD2, // forbidden opcode
    DCP_izy  = 0xD3, // forbidden opcode
    NOP_zpx5 = 0xD4, // forbidden opcode
    CMP_zpx  = 0xD5,
    DEC_zpx  = 0xD6,
    DCP_zpx  = 0xD7, // forbidden opcode
    CLD      = 0xD8,
    CMP_aby  = 0xD9,
    NOP5     = 0xDA, // forbidden opcode
    DCP_aby  = 0xDB, // forbidden opcode
    NOP_abx5 = 0xDC, // forbidden opcode
    CMP_abx  = 0xDD,
    DEC_abx  = 0xDE,
    DCP_abx  = 0xDF, // forbidden opcode
    CPX_imm  = 0xE0,
    SBC_izx  = 0xE1,
    NOP_imm5 = 0xE2, // forbidden opcode
    ISC_izx  = 0xE3, // forbidden opcode
    CPX_zp   = 0xE4,
    SBC_zp   = 0xE5,
    INC_zp   = 0xE6,
    ISC_zp   = 0xE7, // forbidden opcode
    INX      = 0xE8,
    SBC_imm  = 0xE9,
    NOP      = 0xEA,
    SBC_imm2 = 0xEB, // forbidden opcode
    CPX      = 0xEC,
    SBC_abs  = 0xED,
    INC_abs  = 0xEE,
    ISC_abs  = 0xEF, // forbidden opcode
    BEQ_rel  = 0xF0,
    SBC_izy  = 0xF1,
    HLT11    = 0xF2, // forbidden opcode
    ISC_izy  = 0xF3, // forbidden opcode
    NOP_zpx6 = 0xF4, // forbidden opcode
    SBC_zpx  = 0xF5,
    INC_zpx  = 0xF6,
    ISC_zpx  = 0xF7, // forbidden opcode
    SED      = 0xF8,
    SBC_aby  = 0xF9,
    NOP6     = 0xFA, // forbidden opcode
    ISC_aby  = 0xFB, // forbidden opcode
    NOP_abx6 = 0xFC, // forbidden opcode
    SBC_abx  = 0xFD,
    INC_abx  = 0xFE,
    ISC_abx  = 0xFF // forbidden opcode
}


// fetch operand address 
fn get_operand_addr(mode: AddrMode, cpu: &mut cpu::CPU) -> u16
{
    match mode
    {
        AddrMode::Implied           => panic!("Trying to fetch operand addr in implied addr mode."),
        AddrMode::Accumulator       => panic!("Trying to fetch operand addr in accumulator addr mode."),
        AddrMode::Immediate         => panic!("Trying to fetch operand addr in immediate addr mode."),
        AddrMode::Absolute          => cpu.next_word(),
        AddrMode::IndexedAbsoluteX  => {
            let nw = cpu.next_word();
            cpu.mem.read_word_le(nw) + cpu.X as u16 },
        AddrMode::IndexedAbsoluteY  => {
            let nw = cpu.next_word();
            cpu.mem.read_word_le(nw) + cpu.Y as u16 },
        AddrMode::Zeropage          => cpu.next_byte() as u16,
        AddrMode::ZeropageIndexedX  => {
            let nb = cpu.next_byte();
            cpu.mem.read_word_le(nb as u16) + cpu.X as u16 },
        AddrMode::ZeropageIndexedY  => {
            let nb = cpu.next_byte();
            cpu.mem.read_word_le(nb as u16) + cpu.Y as u16 },
        AddrMode::Relative          => {
            let offset: i8 = cpu.next_byte() as i8;
            (cpu.PC as i16 + offset as i16) as u16 },
        AddrMode::AbsoluteIndirect  => panic!("abs_ind not implemented"),
        AddrMode::IndexedIndirectX  => {
            let nb = cpu.next_byte();
            cpu.mem.read_word_le(nb as u16) + cpu.X as u16 },
        AddrMode::IndirectIndexedY  => {
            let nb = cpu.next_byte();
            let addr = cpu.mem.read_word_le(nb as u16);
            cpu.mem.read_word_le(addr) + cpu.Y as u16 },                        
    }    
}

// fetch operand value
pub fn get_operand(mode: AddrMode, cpu: &mut cpu::CPU) -> u8
{
    match mode
    {
        AddrMode::Implied     => panic!("Trying to fetch operand in implied addr mode."),
        AddrMode::Accumulator => cpu.A,
        AddrMode::Immediate   => cpu.next_byte(),
        _ => {
            let opAddr = get_operand_addr(mode, cpu);
            cpu.mem.read_byte(opAddr)
        }
    }    
}

// set operand value
pub fn set_operand(mode: AddrMode, cpu: &mut cpu::CPU, value: u8)
{
    match mode
    {
        AddrMode::Implied     => panic!("Trying to set operand in implied addr mode."),        
        AddrMode::Accumulator => cpu.A = value,
        AddrMode::Immediate   => panic!("Trying to set operand in immediate addr mode."),
        AddrMode::Relative    => panic!("Trying to set operand in relative addr mode."),
        _ => {
            let opAddr = get_operand_addr(mode, cpu);
            cpu.mem.write_byte(opAddr, value)
        }
    }
}
