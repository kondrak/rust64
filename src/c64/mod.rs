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
    cpu: cpu::CPU,
    vic: vic::VICShared,

    cycle_count: u32,
}

impl C64
{
    pub fn new(renderer: &sdl2::render::Renderer) -> C64
    {
        let memory : memory::MemShared = memory::Memory::new_shared();
        let vic : vic::VICShared = vic::VIC::new_shared(memory.clone(), renderer);
        C64
        {
            memory: memory.clone(),                     // shared system memory (RAM, ROM, IO registers)
            //clock: clock::Clock::new(),
            cpu: cpu::CPU::new(memory.clone(), vic.clone()),
            vic: vic.clone(),
            cycle_count: 0,
        }
    }

    pub fn reset(&mut self)
    {
        self.memory.borrow_mut().reset();
        self.cpu.reset();
    }
    
    
    pub fn update(&mut self)
    {
        //if self.clock.tick() { println!("Clock tick"); }
        self.vic.borrow_mut().update();
        // update sid here when it's done
        self.cpu.update();

        self.cycle_count += 1;
    }

    pub fn vblank(&self, draw_frame: bool)
    {
        // TODO
    }

    // debug
    pub fn render(&self, renderer: &mut sdl2::render::Renderer)
    {
        self.vic.borrow_mut().render(renderer);
    }
}
