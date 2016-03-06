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
mod vic_tables;
mod sid;
mod sid_tables;

pub const SCREEN_WIDTH:  usize = 384; // extend 20 pixels left and right for the borders
pub const SCREEN_HEIGHT: usize = 272; // extend 36 pixels top and down for the borders

// PAL clock frequency in Hz
const CLOCK_FREQ: f64 = 1.5 * 985248.0;


pub struct C64 {
    pub window: minifb::Window,
    memory: memory::MemShared,
    io:    io::IO,
    clock: clock::Clock,
    cpu: cpu::CPUShared,
    cia1: cia::CIAShared,
    cia2: cia::CIAShared,
    vic: vic::VICShared,
    sid: sid::SIDShared,

    debugger: Option<debugger::Debugger>,
    boot_complete: bool,
    pub file_to_load: String,
    cycle_count: u32,
}

impl C64 {
    pub fn new(window_scale: Scale, debugger_on: bool) -> C64 {
        let memory = memory::Memory::new_shared();
        let vic    = vic::VIC::new_shared();
        let cia1   = cia::CIA::new_shared(true);
        let cia2   = cia::CIA::new_shared(false);
        let cpu    = cpu::CPU::new_shared();
        let sid    = sid::SID::new_shared();

        let mut c64 = C64 {
            window: Window::new("Rust64", SCREEN_WIDTH, SCREEN_HEIGHT, WindowOptions { scale: window_scale, ..Default::default() }).unwrap(),
            memory: memory.clone(), // shared system memory (RAM, ROM, IO registers)
            io:    io::IO::new(),
            clock: clock::Clock::new(CLOCK_FREQ),
            cpu:   cpu.clone(),
            cia1:  cia1.clone(),
            cia2:  cia2.clone(),
            vic:   vic.clone(),
            sid:   sid.clone(),
            debugger: if debugger_on { Some(debugger::Debugger::new()) } else { None },
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
        c64.sid.borrow_mut().set_references(memory.clone());
        c64.cpu.borrow_mut().set_references(memory.clone(), vic.clone(), cia1.clone(), cia2.clone(), sid.clone());
        
        drop(memory);
        drop(cia1);
        drop(cia2);
        drop(vic);
        drop(cpu);
        drop(sid);
        
        c64
    }

    pub fn reset(&mut self) {
        self.memory.borrow_mut().reset();
        self.cpu.borrow_mut().reset();
        self.cia1.borrow_mut().reset();
        self.cia2.borrow_mut().reset();
        self.sid.borrow_mut().reset();
    }
    

    fn load_prg(&mut self, filename: &str) {
        let prg_data = utils::open_file(filename, 0);
        let start_address: u16 = ((prg_data[1] as u16) << 8) | (prg_data[0] as u16);
        println!("Loading {} to start location at ${:04x} ({})", filename, start_address, start_address);

        for i in 2..(prg_data.len()) {
            self.memory.borrow_mut().write_byte(start_address + (i as u16) - 2, prg_data[i]);
        }
    }

    
    pub fn run(&mut self) {
        if !self.boot_complete {
            // $A480 is the BASIC warm start sequence - safe to assume we can load a cmdline program now
            self.boot_complete = self.cpu.borrow_mut().PC == 0xA480;
 
            if self.boot_complete {
                let prg_file = &self.file_to_load.to_owned()[..];
                
                if prg_file.len() > 0 {
                    self.boot_complete = true; self.load_prg(prg_file);
                }
            }
        }
        
        if self.clock.tick() {
            let mut should_trigger_vblank = false;

            if self.vic.borrow_mut().update(self.cycle_count, &mut should_trigger_vblank) {
                self.sid.borrow_mut().update();
            }

            self.cia1.borrow_mut().process_irq();
            self.cia2.borrow_mut().process_irq();
            self.cia1.borrow_mut().update();
            self.cia2.borrow_mut().update();
        
            self.cpu.borrow_mut().update(self.cycle_count);

            match self.debugger {
                Some(ref mut dbg) => {
                    dbg.update_raster_window(&mut self.vic);
                    if should_trigger_vblank {
                        dbg.render(&mut self.cpu, &mut self.memory);
                    }
                },
                None => (),
            }

            if should_trigger_vblank {
                self.window.update_with_buffer(&self.vic.borrow_mut().window_buffer);
                self.io.update(&self.window, &mut self.cia1);
                self.cia1.borrow_mut().count_tod();
                self.cia2.borrow_mut().count_tod();

                if self.io.check_restore_key(&self.window) {
                    self.cpu.borrow_mut().set_nmi(true);
                }
            }

            if self.window.is_key_pressed(Key::F11, KeyRepeat::No) {
                let di = self.cpu.borrow_mut().debug_instr;
                self.cpu.borrow_mut().debug_instr = !di;
            }

            if self.window.is_key_pressed(Key::F12, KeyRepeat::No) {
                self.reset();
            }

            self.cycle_count += 1;
        }

        self.sid.borrow_mut().update_audio();
    }
}
