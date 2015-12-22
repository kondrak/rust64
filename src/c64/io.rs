extern crate minifb;
use minifb::*;
use c64::cia;

/*
 C64 keyboard map:

 Bit |    7      6   5     4      3     2        1       0
-----+------------------------------------------------------
  7  |   STOP    Q   C=  SPACE    2    CTRL     <-       1
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

    pub fn update_keystates(&mut self, window: &Window, cia1: &mut cia::CIAShared)
    {
        let mut c64_keycode: u8 = 0xFF;
        
        if window.is_key_down(Key::Key0) { c64_keycode = self.keycode_to_c64(Key::Key0); }
        if window.is_key_down(Key::Key1) { c64_keycode = self.keycode_to_c64(Key::Key1); }
        if window.is_key_down(Key::Key2) { c64_keycode = self.keycode_to_c64(Key::Key2); }
        if window.is_key_down(Key::Key3) { c64_keycode = self.keycode_to_c64(Key::Key3); }
        if window.is_key_down(Key::Key4) { c64_keycode = self.keycode_to_c64(Key::Key4); }
        if window.is_key_down(Key::Key5) { c64_keycode = self.keycode_to_c64(Key::Key5); }
        if window.is_key_down(Key::Key6) { c64_keycode = self.keycode_to_c64(Key::Key6); }
        if window.is_key_down(Key::Key7) { c64_keycode = self.keycode_to_c64(Key::Key7); }
        if window.is_key_down(Key::Key8) { c64_keycode = self.keycode_to_c64(Key::Key8); }
        if window.is_key_down(Key::Key9) { c64_keycode = self.keycode_to_c64(Key::Key9); }
        
        if window.is_key_down(Key::A) { c64_keycode = self.keycode_to_c64(Key::A); }
        if window.is_key_down(Key::B) { c64_keycode = self.keycode_to_c64(Key::B); }
        if window.is_key_down(Key::C) { c64_keycode = self.keycode_to_c64(Key::C); }
        if window.is_key_down(Key::D) { c64_keycode = self.keycode_to_c64(Key::D); }
        if window.is_key_down(Key::E) { c64_keycode = self.keycode_to_c64(Key::E); }
        if window.is_key_down(Key::F) { c64_keycode = self.keycode_to_c64(Key::F); }
        if window.is_key_down(Key::G) { c64_keycode = self.keycode_to_c64(Key::G); }
        if window.is_key_down(Key::H) { c64_keycode = self.keycode_to_c64(Key::H); }
        if window.is_key_down(Key::I) { c64_keycode = self.keycode_to_c64(Key::I); }
        if window.is_key_down(Key::J) { c64_keycode = self.keycode_to_c64(Key::J); }
        if window.is_key_down(Key::K) { c64_keycode = self.keycode_to_c64(Key::K); }
        if window.is_key_down(Key::L) { c64_keycode = self.keycode_to_c64(Key::L); }
        if window.is_key_down(Key::M) { c64_keycode = self.keycode_to_c64(Key::M); }
        if window.is_key_down(Key::N) { c64_keycode = self.keycode_to_c64(Key::N); }
        if window.is_key_down(Key::O) { c64_keycode = self.keycode_to_c64(Key::O); }
        if window.is_key_down(Key::P) { c64_keycode = self.keycode_to_c64(Key::P); }
        if window.is_key_down(Key::Q) { c64_keycode = self.keycode_to_c64(Key::Q); }
        if window.is_key_down(Key::R) { c64_keycode = self.keycode_to_c64(Key::R); }
        if window.is_key_down(Key::S) { c64_keycode = self.keycode_to_c64(Key::S); }
        if window.is_key_down(Key::T) { c64_keycode = self.keycode_to_c64(Key::T); }
        if window.is_key_down(Key::U) { c64_keycode = self.keycode_to_c64(Key::U); }
        if window.is_key_down(Key::V) { c64_keycode = self.keycode_to_c64(Key::V); }
        if window.is_key_down(Key::W) { c64_keycode = self.keycode_to_c64(Key::W); }
        if window.is_key_down(Key::X) { c64_keycode = self.keycode_to_c64(Key::X); }
        if window.is_key_down(Key::Y) { c64_keycode = self.keycode_to_c64(Key::Y); }
        if window.is_key_down(Key::Z) { c64_keycode = self.keycode_to_c64(Key::Z); }

        if window.is_key_down(Key::F1) { c64_keycode = self.keycode_to_c64(Key::F1); }
        if window.is_key_down(Key::F2) { c64_keycode = self.keycode_to_c64(Key::F2); }
        if window.is_key_down(Key::F3) { c64_keycode = self.keycode_to_c64(Key::F3); }
        if window.is_key_down(Key::F4) { c64_keycode = self.keycode_to_c64(Key::F4); }
        if window.is_key_down(Key::F5) { c64_keycode = self.keycode_to_c64(Key::F5); }
        if window.is_key_down(Key::F6) { c64_keycode = self.keycode_to_c64(Key::F6); }
        if window.is_key_down(Key::F7) { c64_keycode = self.keycode_to_c64(Key::F7); }
        if window.is_key_down(Key::F8) { c64_keycode = self.keycode_to_c64(Key::F8); }
        
        if window.is_key_down(Key::Down)  { c64_keycode = self.keycode_to_c64(Key::Down); }
        if window.is_key_down(Key::Up)    { c64_keycode = self.keycode_to_c64(Key::Up);   }
        if window.is_key_down(Key::Right) { c64_keycode = self.keycode_to_c64(Key::Right); }
        if window.is_key_down(Key::Left)  { c64_keycode = self.keycode_to_c64(Key::Left);  }
        if window.is_key_down(Key::Space) { c64_keycode = self.keycode_to_c64(Key::Space);  }
        if window.is_key_down(Key::Comma) { c64_keycode = self.keycode_to_c64(Key::Comma);  }
        if window.is_key_down(Key::Period) { c64_keycode = self.keycode_to_c64(Key::Period); }
        if window.is_key_down(Key::Slash)  { c64_keycode = self.keycode_to_c64(Key::Slash);  }
        if window.is_key_down(Key::NumPadAsterisk)  { c64_keycode = self.keycode_to_c64(Key::NumPadAsterisk); }
        if window.is_key_down(Key::Backspace) { c64_keycode = self.keycode_to_c64(Key::Backspace); }
        if window.is_key_down(Key::Minus)  { c64_keycode = self.keycode_to_c64(Key::Minus); }
        //if window.is_key_down(Key::Plus)   { c64_keycode = self.keycode_to_c64(Key::Plus); }
        if window.is_key_down(Key::Insert) { c64_keycode = self.keycode_to_c64(Key::Insert); }
        if window.is_key_down(Key::Home)   { c64_keycode = self.keycode_to_c64(Key::Home); }
        if window.is_key_down(Key::LeftBracket)  { c64_keycode = self.keycode_to_c64(Key::LeftBracket); }
        if window.is_key_down(Key::RightBracket) { c64_keycode = self.keycode_to_c64(Key::RightBracket); }
        if window.is_key_down(Key::Delete) { c64_keycode = self.keycode_to_c64(Key::Delete); }

        if window.is_key_down(Key::Semicolon)  { c64_keycode = self.keycode_to_c64(Key::Semicolon);  }
        if window.is_key_down(Key::Apostrophe) { c64_keycode = self.keycode_to_c64(Key::Apostrophe); }
        if window.is_key_down(Key::Backslash)  { c64_keycode = self.keycode_to_c64(Key::Backslash);  }
        if window.is_key_down(Key::Tab)        { c64_keycode = self.keycode_to_c64(Key::Tab);        }
        if window.is_key_down(Key::LeftCtrl)   { c64_keycode = self.keycode_to_c64(Key::LeftCtrl);   }

        if c64_keycode != 0xFF
        {
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
        // iterating over all keys is crawling-slow for some reason...
       /* for key in window.get_keys().unwrap()
        {
            match key {
                minifb::Key::A => println!("a"),
                minifb::Key::B => println!("b"),
                _ => (), }
        } */
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
            Key::Backspace => to_c64(0, 0),
            // Plus key
            Key::Minus  => to_c64(5, 0),
            // Minus key
            //Key::Plus   => to_c64(5, 3),
            // Pound key
            Key::Insert => to_c64(6, 0),
            // CLR/Home key
            Key::Home => to_c64(6, 3),
            // @ key
            Key::LeftBracket  => to_c64(5, 6),
            // Asterisk key
            Key::RightBracket => to_c64(6, 1),
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
