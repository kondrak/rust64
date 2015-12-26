extern crate sdl2;
extern crate minifb;
use minifb::*;
use utils;
pub mod cpu;
pub mod opcodes;
mod clock;
mod memory;
mod io;
mod cia;
mod vic;

pub const SCREEN_WIDTH:  usize = 384; // extend 20 pixels left and right for the borders
pub const SCREEN_HEIGHT: usize = 272; // extend 36 pixels top and down for the borders

// PAL clock frequency in Hz
const CLOCK_FREQ: f64 = 985248.0;


pub struct C64
{
    pub window: minifb::Window,
    memory: memory::MemShared,
    io:    io::IO,
    clock: clock::Clock,
    cpu: cpu::CPUShared,
    cia1: cia::CIAShared,
    cia2: cia::CIAShared,
    vic: vic::VICShared,

    boot_complete: bool,
    pub file_to_load: String,
    cycle_count: u32,
}

impl C64
{
    pub fn new() -> C64
    {
        let memory = memory::Memory::new_shared();
        let vic    = vic::VIC::new_shared();
        let cia1   = cia::CIA::new_shared(true);
        let cia2   = cia::CIA::new_shared(false);
        let cpu    = cpu::CPU::new_shared();

        let c64 = C64
        {
            window: Window::new("Rust64", SCREEN_WIDTH, SCREEN_HEIGHT, Scale::X1, Vsync::No).unwrap(),
            memory: memory.clone(), // shared system memory (RAM, ROM, IO registers)
            io:    io::IO::new(),
            clock: clock::Clock::new(CLOCK_FREQ),
            cpu: cpu.clone(),
            cia1: cia1.clone(),
            cia2: cia2.clone(),
            vic: vic.clone(),
            boot_complete: false,
            file_to_load: String::new(),
            cycle_count: 0,
        };

        // cyclic dependencies are not possible in Rust (yet?), so we have
        // to resort to setting references manually
        c64.cia1.borrow_mut().set_references(memory.clone(), cpu.clone(), vic.clone());
        c64.cia2.borrow_mut().set_references(memory.clone(), cpu.clone(), vic.clone());
        c64.vic.borrow_mut().set_references(memory.clone(), cpu.clone());
        c64.cpu.borrow_mut().set_references(memory.clone(), vic.clone(), cia1.clone(), cia2.clone());
        
        drop(memory);
        drop(cia1);
        drop(cia2);
        drop(vic);
        drop(cpu);
        
        c64
    }

    pub fn reset(&mut self)
    {
        self.memory.borrow_mut().reset();
        self.cpu.borrow_mut().reset();
        self.cia1.borrow_mut().reset();
        self.cia2.borrow_mut().reset();
    }
    

    fn load_prg(&mut self, filename: &str)
    {
        let prg_data = utils::open_file(filename, 0);
        let start_address: u16 = ((prg_data[1] as u16) << 8) | (prg_data[0] as u16);
        //let end_addr = start_address + (prg_data.len() as u16) - 2;
        println!("Loading {} to start location at ${:04x} ({})", filename, start_address, start_address);

        for i in 2..(prg_data.len())
        {
            self.memory.borrow_mut().write_byte(start_address + (i as u16) - 2, prg_data[i]);
        }

        //self.memory.borrow_mut().write_word_le(0x02b, start_address);
        //self.memory.borrow_mut().write_word_le(0x02d, end_addr);
        //self.memory.borrow_mut().write_word_le(0x02f, end_addr);
        //self.memory.borrow_mut().write_word_le(0x031, end_addr);
    }

    
    pub fn run(&mut self)
    {
        if !self.boot_complete
        {
            // $A480 is the BASIC warm start sequence - safe to assume we can load a cmdline program now
            self.boot_complete = self.cpu.borrow_mut().PC == 0xA480;

            if self.boot_complete
            {
                if self.file_to_load.len() > 0
                {
                    let prg_file = &self.file_to_load.to_owned()[..];
                    self.load_prg(prg_file);
                }
            }
        }
        
        if self.clock.tick() { 
            let mut should_trigger_vblank = false;

            self.vic.borrow_mut().update(self.cycle_count, &mut should_trigger_vblank);

            // TODO: update sid *HERE* when it's done
            self.cia1.borrow_mut().process_irq();
            self.cia2.borrow_mut().process_irq();
            self.cia1.borrow_mut().update();
            self.cia2.borrow_mut().update();
        
            self.cpu.borrow_mut().update();

            if should_trigger_vblank
            {
                self.window.update(&self.vic.borrow_mut().window_buffer);
                self.io.update(&self.window, &mut self.cia1);
                self.cia1.borrow_mut().count_tod();
                self.cia2.borrow_mut().count_tod();

                if self.io.check_restore_key(&self.window)
                {
                    self.cpu.borrow_mut().trigger_nmi();
                }
            }
            
            self.cycle_count += 1;
        }
    }
}
