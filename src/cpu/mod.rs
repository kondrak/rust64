// The CPU
mod opcodes;
    
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
            X: StatusFlag::X as u8,
            Y: 0
        }        
    }
    
    pub fn reset(&self)
    {
        // set the registers to initial state on power up
    }

    pub fn update(&self)
    {
        // process opcodes, to the cpu stuff
    }     

    fn process_op(opcode: u8)
    {
        match opcode
        {
            0x00 => println!("TODO: {}", opcode),
            _ => println!("opcode")
        }        
    }
}
