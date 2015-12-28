use utils;

pub struct SysFont
{
    data: Vec<u8>,
    window_w: usize,
    window_h: usize
}


impl SysFont
{
    pub fn new(window_w: usize, window_h: usize) -> SysFont
    {
        let mut font = SysFont
        {
            data: Vec::<u8>::new(),
            window_w: window_w,
            window_h: window_h
        };

        let bmp_data = utils::open_file("res/font.bmp", 54);

        let mut j: i32 = 256*63*3;
        let mut i;

        while j >= 0
        {
            i = 0;
            while i < 256 * 3
            {
                let color = if bmp_data[i + j as usize] != 0 { 1 } else { 0 };
                font.data.push(color);
                i+= 3;
            }
            
            j -= 256 * 3;
        }
        
        font
    }

    pub fn draw_text_rgb(&self, window_buffer: &mut Vec<u32>, x: usize, y: usize, text: &str, color: u32)
    {
        let chars: Vec<char> = text.chars().collect();
        for i in 0..text.len()
        {
            self.draw_char_rgb(window_buffer, x*8 + 8*i as usize, y*8 as usize, self.ascii_to_petscii(chars[i]), color);
        }
    }    
    
    pub fn draw_text(&self, window_buffer: &mut Vec<u32>, x: usize, y: usize, text: &str, c64_color: u8)
    {
        let chars: Vec<char> = text.chars().collect();
        for i in 0..text.len()
        {
            self.draw_char(window_buffer, x*8 + 8*i as usize, y*8 as usize, self.ascii_to_petscii(chars[i]), c64_color);
        }
    }
    
    pub fn draw_char(&self, window_buffer: &mut Vec<u32>, x: usize, y: usize, charcode: u8, c64_color: u8)
    {
        self.draw_char_rgb(window_buffer, x, y, charcode, utils::fetch_c64_color_rgba(c64_color));
    }

    pub fn draw_char_rgb(&self, window_buffer: &mut Vec<u32>, x: usize, y: usize, charcode: u8, color: u32)
    {
        let char_w: i32 = 8;
        let char_h: i32 = 8;
        let data_x = char_w * (charcode % 32) as i32;
        let data_y = char_h * (charcode / 32) as i32;
        let data_w = data_x + char_w;
        let data_h = data_y + char_h;

        let mut k = 0;
        let mut l = 0;
        for i in data_y..data_h
        {
            for j in data_x..data_w
            {
                window_buffer[x + l + y*self.window_w + k*self.window_w] = self.data[j as usize + (i * 256) as usize] as u32 * color;
                l += 1;
            }
            l = 0;
            k += 1;
        }
    }

    fn ascii_to_petscii(&self, c_ascii: char) -> u8
    {
        match c_ascii
        {
            '@' => 0,
            'A' | 'a' => 1,
            'B' | 'b' => 2,
            'C' | 'c' => 3,
            'D' | 'd' => 4,
            'E' | 'e' => 5,
            'F' | 'f' => 6,
            'G' | 'g' => 7,
            'H' | 'h' => 8,
            'I' | 'i' => 9,
            'J' | 'j' => 10,
            'K' | 'k' => 11,
            'L' | 'l' => 12,
            'M' | 'm' => 13,
            'N' | 'n' => 14,
            'O' | 'o' => 15,
            'P' | 'p' => 16,
            'Q' | 'q' => 17,
            'R' | 'r' => 18,
            'S' | 's' => 19,
            'T' | 't' => 20,
            'U' | 'u' => 21,
            'V' | 'v' => 22,
            'W' | 'w' => 23,
            'X' | 'x' => 24,
            'Y' | 'y' => 25,
            'Z' | 'z'=> 26,
            '[' => 27,
            ']' => 28,
            ' ' => 32,
            '!' => 33,
            '"' => 34,
            '#' => 35,
            '$' => 36,
            '%' => 37,
            '&' => 38,
            '`' => 39,
            '(' => 40,
            ')' => 41,
            '*' => 42,
            '+' => 43,
            ',' => 44,
            '-' => 45,
            '.' => 46,
            '/' => 47,
            '0' => 48,
            '1' => 49,
            '2' => 50,
            '3' => 51,
            '4' => 52,
            '5' => 53,
            '6' => 54,
            '7' => 55,
            '8' => 56,
            '9' => 57,
            ':' => 58,
            ';' => 59,
            '<' => 60,
            '=' => 61,
            '>' => 62,
            '?' => 63,
            _ => 63
        }
    }
}
