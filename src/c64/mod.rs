extern crate sdl2;
extern crate minifb;
use minifb::*;
pub mod cpu;
pub mod opcodes;
//mod clock;
mod memory;
mod io;
mod cia;
mod vic;

pub const SCREEN_WIDTH:  usize = 384; // extend 20 pixels left and right for the borders
pub const SCREEN_HEIGHT: usize = 272; // extend 36 pixels top and down for the borders


pub struct C64
{
    pub window: minifb::Window,
    memory: memory::MemShared,
    keyboard: io::Keyboard,
    //clock: clock::Clock,
    cpu: cpu::CPUShared,
    cia1: cia::CIAShared,
    cia2: cia::CIAShared,
    vic: vic::VICShared,

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
            keyboard: io::Keyboard::new(),
            //clock: clock::Clock::new(),
            cpu: cpu.clone(),
            cia1: cia1.clone(),
            cia2: cia2.clone(),
            vic: vic.clone(),
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
    
    
    pub fn run(&mut self)
    {
        let mut should_trigger_vblank = false;
        //if self.clock.tick() { println!("Clock tick"); }

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
            self.keyboard.update_keystates(&self.window, &mut self.cia1);
            self.cia1.borrow_mut().count_tod();
            self.cia2.borrow_mut().count_tod();

            if self.keyboard.check_restore_key(&self.window)
            {
                self.cpu.borrow_mut().trigger_nmi();
            }
        }
        
        self.cycle_count += 1;
    }
}
