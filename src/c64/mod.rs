extern crate sdl2;
pub mod cpu;
pub mod opcodes;
//mod clock;
mod memory;
mod vic;

pub struct C64
{
    memory: memory::MemShared,
    //clock: clock::Clock,
    cpu: cpu::CPUShared,
    vic: vic::VICShared,

    cycle_count: u32,
}

impl C64
{
    pub fn new(renderer: &sdl2::render::Renderer) -> C64
    {
        let memory = memory::Memory::new_shared();
        let vic    = vic::VIC::new_shared();
        let cpu    = cpu::CPU::new_shared();

        let mut c64 = C64
        {
            memory: memory.clone(),                     // shared system memory (RAM, ROM, IO registers)
            //clock: clock::Clock::new(),
            cpu: cpu.clone(),
            vic: vic.clone(),
            cycle_count: 0,
        };

        // cyclic dependencies are not possible in Rust (yet?), so we have
        // to resort to setting references manually
        c64.vic.borrow_mut().set_references(memory.clone(), cpu.clone());
        c64.cpu.borrow_mut().set_references(memory.clone(), vic.clone());
        
        drop(memory);
        drop(vic);
        drop(cpu);
        
        c64
    }

    pub fn reset(&mut self)
    {
        self.memory.borrow_mut().reset();
        self.cpu.borrow_mut().reset();
    }
    
    
    pub fn update(&mut self)
    {
        //if self.clock.tick() { println!("Clock tick"); }
        self.vic.borrow_mut().update();
        // update sid here when it's done
        self.cpu.borrow_mut().update();

        self.cycle_count += 1;
    }

    pub fn vblank(&self, draw_frame: bool)
    {
        // TODO
    }

    // debug
    /*pub fn render(&self, renderer: &mut sdl2::render::Renderer)
    {
        self.vic.borrow_mut().render(renderer);
    }*/
}
