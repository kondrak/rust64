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


        if window.is_key_down(Key::Key0) { println!("0"); }
        if window.is_key_down(Key::Key1) { println!("1"); }
        if window.is_key_down(Key::Key2) { println!("2"); }
        if window.is_key_down(Key::Key3) { println!("3"); }
        if window.is_key_down(Key::Key4) { println!("4"); }
        if window.is_key_down(Key::Key5) { println!("5"); }
        if window.is_key_down(Key::Key6) { println!("6"); }
        if window.is_key_down(Key::Key7) { println!("7"); }
        if window.is_key_down(Key::Key8) { println!("8"); }
        if window.is_key_down(Key::Key9) { println!("9"); }
        
        if window.is_key_down(Key::A) { println!("A"); }
        if window.is_key_down(Key::B) { println!("B"); }
        if window.is_key_down(Key::C) { println!("C"); }
        if window.is_key_down(Key::D) { println!("D"); }
        if window.is_key_down(Key::E) { println!("E"); }
        if window.is_key_down(Key::F) { println!("F"); }
        if window.is_key_down(Key::G) { println!("G"); }
        if window.is_key_down(Key::H) { println!("H"); }
        if window.is_key_down(Key::I) { println!("I"); }
        if window.is_key_down(Key::J) { println!("J"); }
        if window.is_key_down(Key::K) { println!("K"); }
        if window.is_key_down(Key::L) { println!("L"); }
        if window.is_key_down(Key::M) { println!("M"); }
        if window.is_key_down(Key::N) { println!("N"); }
        if window.is_key_down(Key::O) { println!("O"); }
        if window.is_key_down(Key::P) { println!("P"); }
        if window.is_key_down(Key::Q) { println!("Q"); }
        if window.is_key_down(Key::R) { println!("R"); }
        if window.is_key_down(Key::S) { println!("S"); }
        if window.is_key_down(Key::T) { println!("T"); }
        if window.is_key_down(Key::U) { println!("U"); }
        if window.is_key_down(Key::V) { println!("V"); }
        if window.is_key_down(Key::W) { println!("W"); }
        if window.is_key_down(Key::X) { println!("X"); }
        if window.is_key_down(Key::Y) { println!("Y"); }
        if window.is_key_down(Key::Z) { println!("Z"); }

        if window.is_key_down(Key::F1) { println!("F1"); }
        if window.is_key_down(Key::F2) { println!("F2"); }
        if window.is_key_down(Key::F3) { println!("F3"); }
        if window.is_key_down(Key::F4) { println!("F4"); }
        if window.is_key_down(Key::F5) { println!("F5"); }
        if window.is_key_down(Key::F6) { println!("F6"); }
        if window.is_key_down(Key::F7) { println!("F7"); }
        if window.is_key_down(Key::F8) { println!("F8"); }   
        
        if window.is_key_down(Key::Down) { println!("Down"); }
        if window.is_key_down(Key::Left) { println!("Left"); }
        if window.is_key_down(Key::Right) { println!("Right"); }
        if window.is_key_down(Key::Up) { println!("Up"); }
        
        // iterating over all keys is crawling-slow for some reason...
       /* for key in window.get_keys().unwrap()
        {
            match key {
                minifb::Key::A => println!("a"),
                minifb::Key::B => println!("b"),
                _ => (), }
        } */
    }
}
