extern crate sdl2;
pub mod cpu;
pub mod opcodes;
pub mod memory;

use video;

pub struct C64
{
    memory: memory::MemShared,
    cpu: cpu::CPU,
    font: video::font::SysFont,
}

impl C64
{
    pub fn new(renderer: &sdl2::render::Renderer) -> C64
    {
        let memory : memory::MemShared = memory::Memory::new_shared();
        
        C64
        {
            memory: memory.clone(),                     // shared system memory (RAM, ROM, IO registers)
            cpu: cpu::CPU::new(memory.clone()),
            font: video::font::SysFont::new(renderer),
        }
    }

    pub fn reset(&mut self)
    {
        self.memory.borrow_mut().reset();
        self.cpu.reset();
    }
    
    
    pub fn update(&mut self)
    {
        self.cpu.update();
    }


    pub fn render(&mut self, renderer: &mut sdl2::render::Renderer)
    {
        // dump screen memory
        let mut start = 0x0400;

        for y in 0..25
        {
            for x in 0..40
            {
                let d = self.memory.borrow_mut().read_byte(start);
                self.font.draw_char(renderer, x, y, d);
                start += 1;
            }
        }
    }
}
