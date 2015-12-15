extern crate minifb;
use minifb::*;

// C64 keyboard
pub struct Keyboard
{
    keys: u8,
}

impl Keyboard
{
    pub fn new() -> Keyboard
    {
        Keyboard
        {
            keys: 0,
        }
    }

    pub fn update_keystates(&mut self, window: &Window)
    {
        // TODO: actual key state saving should likely be done outside of vsync;
        // only update CIA key matrix in vstate
        for key in window.get_keys().unwrap()
        {
            match key {
                minifb::Key::A => println!("a"),
                minifb::Key::B => println!("b"),
                _ => (), }
        }
    }
}
