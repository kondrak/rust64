//extern crate sdl2;
extern crate minifb;
use minifb::*;
use utils;
use debugger;
pub mod cpu;
pub mod opcodes;
mod clock;
pub mod memory;
mod io;
mod cia;
pub mod vic;
mod sid;

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
    sid: sid::SID,

    debugger: debugger::Debugger,
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

        let mut c64 = C64
        {
            window: Window::new("Rust64", SCREEN_WIDTH, SCREEN_HEIGHT, WindowOptions::default()).unwrap(),
            memory: memory.clone(), // shared system memory (RAM, ROM, IO registers)
            io:    io::IO::new(),
            clock: clock::Clock::new(CLOCK_FREQ),
            cpu: cpu.clone(),
            cia1: cia1.clone(),
            cia2: cia2.clone(),
            vic: vic.clone(),
            sid: sid::SID::new(),

            debugger: debugger::Debugger::new(),
            boot_complete: false,
            file_to_load: String::new(),
            cycle_count: 0,
        };

        c64.window.set_position(75, 20);

        // cyclic dependencies are not possible in Rust (yet?), so we have
        // to resort to setting references manually
        c64.cia1.borrow_mut().set_references(memory.clone(), cpu.clone(), vic.clone());
        c64.cia2.borrow_mut().set_references(memory.clone(), cpu.clone(), vic.clone());
        c64.vic.borrow_mut().set_references(memory.clone(), cpu.clone());
        c64.sid.set_references(memory.clone());
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
        println!("Loading {} to start location at ${:04x} ({})", filename, start_address, start_address);

        for i in 2..(prg_data.len())
        {
            self.memory.borrow_mut().write_byte(start_address + (i as u16) - 2, prg_data[i]);
        }
    }

    
    pub fn run(&mut self)
    {
        if !self.boot_complete
        {
            // $A480 is the BASIC warm start sequence - safe to assume we can load a cmdline program now
            self.boot_complete = self.cpu.borrow_mut().PC == 0xA480;
 
            if self.boot_complete
            {
                let prg_file = &self.file_to_load.to_owned()[..];
                //let prg_file = "bcs-01.prg";     // ok
                //let prg_file = "triad-01.prg";
                //let prg_file = "dd-01.prg";    // sprites null
                //let prg_file = "flt-01.prg";  // ok - blinking
                //let prg_file = "esi-02.prg";   // ok - blinking
                //let prg_file = "htl-03.prg";
                //let prg_file = "ikari-02.prg"; // ok
                //let prg_file = "img.prg";
                //let prg_file ="tests/bgcolor.prg";
                //let prg_file = "spritedemo.prg";
                //let prg_file ="flapper.prg";
                //let prg_file ="superball.prg";
                //let prg_file = "flt-09.prg";    // bw - blinking
                //let prg_file = "newage-03.prg"; // ok
                //let prg_file = "orion-26.prg";  // ok
                //let prg_file = "energy-09.prg";
                //let prg_file = "jam-10.prg";  // ok
                //let prg_file = "tpi-01.prg";
                //let prg_file = "711-01.prg"; // ok - blinking
                
                if prg_file.len() > 0
                {
                    //if self.window.is_key_pressed(Key::F11, KeyRepeat::No) {
                    self.boot_complete = true; self.load_prg(prg_file);
                    //}
                }
            }
        }
        
        if self.clock.tick()
        {
            let mut should_trigger_vblank = false;

            if self.vic.borrow_mut().update(self.cycle_count, &mut should_trigger_vblank)
            {
                self.sid.update();
            }

            self.cia1.borrow_mut().process_irq();
            self.cia2.borrow_mut().process_irq();
            self.cia1.borrow_mut().update();
            self.cia2.borrow_mut().update();
        
            self.cpu.borrow_mut().update(self.cycle_count);

            self.debugger.update_raster_window(&mut self.vic);

            if should_trigger_vblank
            {
                self.debugger.render(&mut self.cpu, &mut self.memory);
                self.window.update_with_buffer(&self.vic.borrow_mut().window_buffer);
                self.io.update(&self.window, &mut self.cia1);
                self.cia1.borrow_mut().count_tod();
                self.cia2.borrow_mut().count_tod();

                if self.io.check_restore_key(&self.window)
                {
                    self.cpu.borrow_mut().set_nmi(true);
                }
            }

            if self.window.is_key_pressed(Key::F10, KeyRepeat::No)
            {
                let di = self.cpu.borrow_mut().debug_instr;
                self.cpu.borrow_mut().debug_instr = !di;
            }

            if self.window.is_key_pressed(Key::F12, KeyRepeat::No)
            {
                self.cpu.borrow_mut().reset();
            }

            self.cycle_count += 1;
        }
    }
}
