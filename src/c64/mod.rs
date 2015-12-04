extern crate sdl2;
extern crate minifb;
pub mod cpu;
pub mod opcodes;
//mod clock;
mod memory;
mod cia;
mod vic;

pub const SCREEN_WIDTH:  usize = 384; // extend 20 pixels left and right for the borders
pub const SCREEN_HEIGHT: usize = 272; // extend 36 pixels top and down for the borders


pub struct C64
{
    memory: memory::MemShared,
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
        let cia1   = cia::CIA::new_shared();
        let cia2   = cia::CIA::new_shared();
        let cpu    = cpu::CPU::new_shared();

        let c64 = C64
        {
            memory: memory.clone(),                     // shared system memory (RAM, ROM, IO registers)
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
    
    
    pub fn update(&mut self)
    {
        let mut should_trigger_vblank = false;
        //if self.clock.tick() { println!("Clock tick"); }
        //self.cia1.borrow_mut().update();
        //self.cia2.borrow_mut().update();
        self.vic.borrow_mut().update(self.cycle_count, &mut should_trigger_vblank);
        // update sid here when it's done
        self.cpu.borrow_mut().update();

        if should_trigger_vblank
        {
            //let w = self.vic.borrow_mut().window_buffer[0];
            //println!("w: {}", w);
            //minifb::update(&self.vic.borrow_mut().window_buffer);
        }
        
        self.cycle_count += 1;
    }

    pub fn vblank(&self, draw_frame: bool)
    {
        // TODO
    }

    // debug
    pub fn render(&mut self) -> bool
    {
        //self.vic.borrow_mut().render();
        //true
        minifb::update(&self.vic.borrow_mut().window_buffer)
    }
}
