extern crate minifb;
use minifb::*;
use c64::cia;

/*
 C64 keyboard map:

 Bit |    7      6   5     4      3     2        1       0
-----+------------------------------------------------------
  7  | RUNSTOP   Q   C=  SPACE    2    CTRL     <-       1
  6  |    /      ^   =   RSHIFT  HOME   ;       *      POUND
  5  |    ,      @   "     .      -     L       P        +
  4  |    N      O   K     M      0     J       I        9
  3  |    V      U   H     B      8     G       Y        7
  2  |    X      T   F     C      6     D       R        5
  1  |  LSHIFT   E   S     Z      4     A       W        3
  0  |  CRSR-DN  F5  F3    F1     F7  CRSR-RT  RETURN  DELETE
*/
pub struct Keyboard
{
    pressed_keys: [bool; 256],
}

impl Keyboard
{
    pub fn new() -> Keyboard
    {
        Keyboard
        {
            pressed_keys: [false; 256]
        }
    }

    pub fn update_keystates(&mut self, window: &Window, cia1: &mut cia::CIAShared)
    {
        // TODO: Restore and Run Stop keys
        if window.is_key_down(Key::Key0) { self.on_key_press(Key::Key0, cia1); } else { self.on_key_release(Key::Key0, cia1); }
        if window.is_key_down(Key::Key1) { self.on_key_press(Key::Key1, cia1); } else { self.on_key_release(Key::Key1, cia1); }
        if window.is_key_down(Key::Key2) { self.on_key_press(Key::Key2, cia1); } else { self.on_key_release(Key::Key2, cia1); }
        if window.is_key_down(Key::Key3) { self.on_key_press(Key::Key3, cia1); } else { self.on_key_release(Key::Key3, cia1); }
        if window.is_key_down(Key::Key4) { self.on_key_press(Key::Key4, cia1); } else { self.on_key_release(Key::Key4, cia1); }
        if window.is_key_down(Key::Key5) { self.on_key_press(Key::Key5, cia1); } else { self.on_key_release(Key::Key5, cia1); }
        if window.is_key_down(Key::Key6) { self.on_key_press(Key::Key6, cia1); } else { self.on_key_release(Key::Key6, cia1); }
        if window.is_key_down(Key::Key7) { self.on_key_press(Key::Key7, cia1); } else { self.on_key_release(Key::Key7, cia1); }
        if window.is_key_down(Key::Key8) { self.on_key_press(Key::Key8, cia1); } else { self.on_key_release(Key::Key8, cia1); }
        if window.is_key_down(Key::Key9) { self.on_key_press(Key::Key9, cia1); } else { self.on_key_release(Key::Key9, cia1); }
        
        if window.is_key_down(Key::A) { self.on_key_press(Key::A, cia1); } else { self.on_key_release(Key::A, cia1); }
        if window.is_key_down(Key::B) { self.on_key_press(Key::B, cia1); } else { self.on_key_release(Key::B, cia1); }
        if window.is_key_down(Key::C) { self.on_key_press(Key::C, cia1); } else { self.on_key_release(Key::C, cia1); }
        if window.is_key_down(Key::D) { self.on_key_press(Key::D, cia1); } else { self.on_key_release(Key::D, cia1); }
        if window.is_key_down(Key::E) { self.on_key_press(Key::E, cia1); } else { self.on_key_release(Key::E, cia1); }
        if window.is_key_down(Key::F) { self.on_key_press(Key::F, cia1); } else { self.on_key_release(Key::F, cia1); }
        if window.is_key_down(Key::G) { self.on_key_press(Key::G, cia1); } else { self.on_key_release(Key::G, cia1); }
        if window.is_key_down(Key::H) { self.on_key_press(Key::H, cia1); } else { self.on_key_release(Key::H, cia1); }
        if window.is_key_down(Key::I) { self.on_key_press(Key::I, cia1); } else { self.on_key_release(Key::I, cia1); }
        if window.is_key_down(Key::J) { self.on_key_press(Key::J, cia1); } else { self.on_key_release(Key::J, cia1); }
        if window.is_key_down(Key::K) { self.on_key_press(Key::K, cia1); } else { self.on_key_release(Key::K, cia1); }
        if window.is_key_down(Key::L) { self.on_key_press(Key::L, cia1); } else { self.on_key_release(Key::L, cia1); }
        if window.is_key_down(Key::M) { self.on_key_press(Key::M, cia1); } else { self.on_key_release(Key::M, cia1); }
        if window.is_key_down(Key::N) { self.on_key_press(Key::N, cia1); } else { self.on_key_release(Key::N, cia1); }
        if window.is_key_down(Key::O) { self.on_key_press(Key::O, cia1); } else { self.on_key_release(Key::O, cia1); }
        if window.is_key_down(Key::P) { self.on_key_press(Key::P, cia1); } else { self.on_key_release(Key::P, cia1); }
        if window.is_key_down(Key::Q) { self.on_key_press(Key::Q, cia1); } else { self.on_key_release(Key::Q, cia1); }
        if window.is_key_down(Key::R) { self.on_key_press(Key::R, cia1); } else { self.on_key_release(Key::R, cia1); }
        if window.is_key_down(Key::S) { self.on_key_press(Key::S, cia1); } else { self.on_key_release(Key::S, cia1); }
        if window.is_key_down(Key::T) { self.on_key_press(Key::T, cia1); } else { self.on_key_release(Key::T, cia1); }
        if window.is_key_down(Key::U) { self.on_key_press(Key::U, cia1); } else { self.on_key_release(Key::U, cia1); }
        if window.is_key_down(Key::V) { self.on_key_press(Key::V, cia1); } else { self.on_key_release(Key::V, cia1); }
        if window.is_key_down(Key::W) { self.on_key_press(Key::W, cia1); } else { self.on_key_release(Key::W, cia1); }
        if window.is_key_down(Key::X) { self.on_key_press(Key::X, cia1); } else { self.on_key_release(Key::X, cia1); }
        if window.is_key_down(Key::Y) { self.on_key_press(Key::Y, cia1); } else { self.on_key_release(Key::Y, cia1); }
        if window.is_key_down(Key::Z) { self.on_key_press(Key::Z, cia1); } else { self.on_key_release(Key::Z, cia1); }

        if window.is_key_down(Key::F1) { self.on_key_press(Key::F1, cia1); } else { self.on_key_release(Key::F1, cia1); }
        if window.is_key_down(Key::F2) { self.on_key_press(Key::F2, cia1); } else { self.on_key_release(Key::F2, cia1); }
        if window.is_key_down(Key::F3) { self.on_key_press(Key::F3, cia1); } else { self.on_key_release(Key::F3, cia1); }
        if window.is_key_down(Key::F4) { self.on_key_press(Key::F4, cia1); } else { self.on_key_release(Key::F4, cia1); }
        if window.is_key_down(Key::F5) { self.on_key_press(Key::F5, cia1); } else { self.on_key_release(Key::F5, cia1); }
        if window.is_key_down(Key::F6) { self.on_key_press(Key::F6, cia1); } else { self.on_key_release(Key::F6, cia1); }
        if window.is_key_down(Key::F7) { self.on_key_press(Key::F7, cia1); } else { self.on_key_release(Key::F7, cia1); }
        if window.is_key_down(Key::F8) { self.on_key_press(Key::F8, cia1); } else { self.on_key_release(Key::F8, cia1); }
        
        if window.is_key_down(Key::Down)   { self.on_key_press(Key::Down, cia1);  } else { self.on_key_release(Key::Down, cia1); }
        if window.is_key_down(Key::Up)     { self.on_key_press(Key::Up, cia1);    } else { self.on_key_release(Key::Up, cia1); }
        if window.is_key_down(Key::Right)  { self.on_key_press(Key::Right, cia1); } else { self.on_key_release(Key::Right, cia1); }
        if window.is_key_down(Key::Left)   { self.on_key_press(Key::Left, cia1);  } else { self.on_key_release(Key::Left, cia1); }
        if window.is_key_down(Key::Space)  { self.on_key_press(Key::Space, cia1); } else { self.on_key_release(Key::Space, cia1); }
        if window.is_key_down(Key::Comma)  { self.on_key_press(Key::Comma, cia1); } else { self.on_key_release(Key::Comma, cia1); }
        if window.is_key_down(Key::Period) { self.on_key_press(Key::Period, cia1); } else { self.on_key_release(Key::Period, cia1); }
        if window.is_key_down(Key::Slash)  { self.on_key_press(Key::Slash, cia1);   } else { self.on_key_release(Key::Slash, cia1); }
        if window.is_key_down(Key::NumPadAsterisk) { self.on_key_press(Key::NumPadAsterisk, cia1); } else { self.on_key_release(Key::NumPadAsterisk, cia1); }
        if window.is_key_down(Key::Enter)      { self.on_key_press(Key::Enter, cia1);      } else { self.on_key_release(Key::Enter, cia1); }
        if window.is_key_down(Key::Backspace)  { self.on_key_press(Key::Backspace, cia1);  } else { self.on_key_release(Key::Backspace, cia1); }
        if window.is_key_down(Key::Backquote)  { self.on_key_press(Key::Backquote, cia1);  } else { self.on_key_release(Key::Backquote, cia1); }
        if window.is_key_down(Key::LeftShift)  { self.on_key_press(Key::LeftShift, cia1);  } else { self.on_key_release(Key::LeftShift, cia1); }
        if window.is_key_down(Key::RightShift) { self.on_key_press(Key::RightShift, cia1); } else { self.on_key_release(Key::RightShift, cia1); }
        //if window.is_key_down(Key::Escape) { self.on_key_press(Key::Escape, cia1); } else { self.on_key_release(Key::Escape, cia1); }
        if window.is_key_down(Key::Minus)  { self.on_key_press(Key::Minus, cia1);  } else { self.on_key_release(Key::Minus, cia1); }
        if window.is_key_down(Key::Equal)  { self.on_key_press(Key::Equal, cia1);  } else { self.on_key_release(Key::Equal, cia1); }
        if window.is_key_down(Key::Insert) { self.on_key_press(Key::Insert, cia1); } else { self.on_key_release(Key::Insert, cia1); }
        if window.is_key_down(Key::Home)   { self.on_key_press(Key::Home, cia1);   } else { self.on_key_release(Key::Home, cia1); }
        if window.is_key_down(Key::LeftBracket)  { self.on_key_press(Key::LeftBracket, cia1);    } else { self.on_key_release(Key::LeftBracket, cia1); }
        if window.is_key_down(Key::RightBracket) { self.on_key_press(Key::NumPadAsterisk, cia1); } else { self.on_key_release(Key::NumPadAsterisk, cia1); }
        if window.is_key_down(Key::Delete) { self.on_key_press(Key::Delete, cia1); } else { self.on_key_release(Key::Delete, cia1); }

        if window.is_key_down(Key::Semicolon)  { self.on_key_press(Key::Semicolon, cia1);  } else { self.on_key_release(Key::Semicolon, cia1); }
        if window.is_key_down(Key::Apostrophe) { self.on_key_press(Key::Apostrophe, cia1); } else { self.on_key_release(Key::Apostrophe, cia1); }
        if window.is_key_down(Key::Backslash)  { self.on_key_press(Key::Backslash, cia1);  } else { self.on_key_release(Key::Backslash, cia1); }
        if window.is_key_down(Key::Tab)        { self.on_key_press(Key::Tab, cia1);        } else { self.on_key_release(Key::Tab, cia1); }
        if window.is_key_down(Key::LeftCtrl)   { self.on_key_press(Key::LeftCtrl, cia1);   } else { self.on_key_release(Key::LeftCtrl, cia1); }
        
        // iterating over all keys is crawling-slow for some reason...
       /* for key in window.get_keys().unwrap()
        {
            match key {
                minifb::Key::A => println!("a"),
                minifb::Key::B => println!("b"),
                _ => (), }
        } */
    }

    fn on_key_press(&mut self, keycode: Key, cia1: &mut cia::CIAShared)
    {
        let c64_keycode = self.keycode_to_c64(keycode);

        if self.pressed_keys[c64_keycode as usize] == true
        {
            return
        }
        
        self.pressed_keys[c64_keycode as usize] = true;

        let c64_bit  = c64_keycode & 7;
        let c64_byte = (c64_keycode >> 3) & 7;
        
        // key is shifted?
        if (c64_keycode & 0x80) != 0
        {
            cia1.borrow_mut().key_matrix[6] &= 0xEF;
            cia1.borrow_mut().rev_matrix[4] &= 0xBF;
        }
        
        cia1.borrow_mut().key_matrix[c64_byte as usize] &= !(1 << c64_bit);
        cia1.borrow_mut().rev_matrix[c64_bit as usize]  &= !(1 << c64_byte);
    }

    fn on_key_release(&mut self, keycode: Key, cia1: &mut cia::CIAShared)
    {
        let c64_keycode = self.keycode_to_c64(keycode);

        if self.pressed_keys[c64_keycode as usize] == false
        {
            return
        }
        
        self.pressed_keys[c64_keycode as usize] = false;

        let c64_bit  = c64_keycode & 7;
        let c64_byte = (c64_keycode >> 3) & 7;
        
        // key is shifted?
        if (c64_keycode & 0x80) != 0
        {
            cia1.borrow_mut().key_matrix[6] |= 0x10;
            cia1.borrow_mut().rev_matrix[4] |= 0x40;
        }
        
        cia1.borrow_mut().key_matrix[c64_byte as usize] |= 1 << c64_bit;
        cia1.borrow_mut().rev_matrix[c64_bit as usize]  |= 1 << c64_byte;
    }
    
    fn keycode_to_c64(&self, keycode: Key) -> u8
    {
        // fetch key's bit combination as represented in C64 keyboard matrix
        let to_c64 = |row_bit: u8, col_bit: u8| (row_bit << 3) | col_bit;
        
        match keycode
        {
            Key::Key0 => to_c64(4, 3),
            Key::Key1 => to_c64(7, 0),
            Key::Key2 => to_c64(7, 3),
            Key::Key3 => to_c64(1, 0),
            Key::Key4 => to_c64(1, 3),
            Key::Key5 => to_c64(2, 0),
            Key::Key6 => to_c64(2, 3),
            Key::Key7 => to_c64(3, 0),
            Key::Key8 => to_c64(3, 3),
            Key::Key9 => to_c64(4, 0),
            Key::A => to_c64(1, 2),
            Key::B => to_c64(3, 4),
            Key::C => to_c64(2, 4),
            Key::D => to_c64(2, 2),
            Key::E => to_c64(1, 6),
            Key::F => to_c64(2, 5),
            Key::G => to_c64(3, 2),
            Key::H => to_c64(3, 5),
            Key::I => to_c64(4, 1),
            Key::J => to_c64(4, 2),
            Key::K => to_c64(4, 5),
            Key::L => to_c64(5, 2),
            Key::M => to_c64(4, 4),
            Key::N => to_c64(4, 7),
            Key::O => to_c64(4, 6),
            Key::P => to_c64(5, 1),
            Key::Q => to_c64(7, 6),
            Key::R => to_c64(2, 1),
            Key::S => to_c64(1, 5),
            Key::T => to_c64(2, 6),
            Key::U => to_c64(3, 6),
            Key::V => to_c64(3, 7),
            Key::W => to_c64(1, 1),
            Key::X => to_c64(2, 7),
            Key::Y => to_c64(3, 1),
            Key::Z => to_c64(1, 4),
            Key::F1 => to_c64(0, 4),
            Key::F2 => to_c64(0, 4) | 0x80,
            Key::F3 => to_c64(0, 5),
            Key::F4 => to_c64(0, 5) | 0x80,
            Key::F5 => to_c64(0, 6),
            Key::F6 => to_c64(0, 6) | 0x80,
            Key::F7 => to_c64(0, 3),
            Key::F8 => to_c64(0, 3) | 0x80,
            Key::Down   => to_c64(0, 7),
            Key::Up     => to_c64(0, 7) | 0x80,
            Key::Right  => to_c64(0, 2),
            Key::Left   => to_c64(0, 2) | 0x80,
            Key::Space  => to_c64(7, 4),
            Key::Comma  => to_c64(5, 7),
            Key::Period => to_c64(5, 4),
            Key::Slash  => to_c64(6, 7),
            Key::NumPadAsterisk  => to_c64(6, 1),
            Key::Enter     => to_c64(0, 1),
            Key::Backspace => to_c64(0, 0),
            // Left arrow key
            Key::Backquote  => to_c64(7, 1),
            Key::LeftShift  => to_c64(1, 7),
            Key::RightShift => to_c64(6, 4),
            // Run Stop key
            //Key::Escape => to_c64(7, 7),
            // Plus key
            Key::Minus  => to_c64(5, 0),
            // Minus key
            Key::Equal  => to_c64(5, 3),
            // Pound key
            Key::Insert => to_c64(6, 0),
            // CLR/Home key
            Key::Home => to_c64(6, 3),
            // @ key
            Key::LeftBracket  => to_c64(5, 6),
            // Home key
            Key::Delete => to_c64(6, 6),
            // Colon key
            Key::Semicolon  => to_c64(5, 5),
            // Semicolon key
            Key::Apostrophe => to_c64(6, 2),
            // Equals key
            Key::Backslash => to_c64(6, 5),
            // Control key
            Key::Tab => to_c64(7, 2),
            // Commodore key
            Key::LeftCtrl => to_c64(7, 5),
            _ => panic!("Unsupported key")
        }
    }
}
