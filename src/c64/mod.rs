extern crate sdl2;
extern crate minifb;
pub mod cpu;
pub mod opcodes;
//mod clock;
mod memory;
mod vic;

pub const SCREEN_WIDTH:  usize = 320;
pub const SCREEN_HEIGHT: usize = 200;


pub struct C64
{
    memory: memory::MemShared,
    //clock: clock::Clock,
    cpu: cpu::CPUShared,
    vic: vic::VICShared,

    cycle_count: u32,

    pub window_buffer: [u32; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl C64
{
    pub fn new() -> C64
    {
        let memory = memory::Memory::new_shared();
        let vic    = vic::VIC::new_shared();
        let cpu    = cpu::CPU::new_shared();

        let c64 = C64
        {
            memory: memory.clone(),                     // shared system memory (RAM, ROM, IO registers)
            //clock: clock::Clock::new(),
            cpu: cpu.clone(),
            vic: vic.clone(),
            cycle_count: 0,
            window_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
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
    pub fn render(&mut self) -> bool
    {
        //self.vic.borrow_mut().render(renderer);

        // dump screen memory
        let mut start = 0x0400;

        for y in 0..25
        {
            for x in 0..40
            {
                let d = self.memory.borrow_mut().read_byte(start);
                //self.font.draw_char(renderer, x, y, d);
                //let muti = self.window_buffer[0];
                self.window_buffer[x + y * SCREEN_WIDTH] = if d != 32 { 0x00FFFFFF } else { 0x000088FF };
                start += 1;
            }
        }
        
        minifb::update(&self.window_buffer)
    }
}
