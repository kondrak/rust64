use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

pub fn open_file(filename: &str) -> Vec<u8>
{
    let path = Path::new(&filename);
    
    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", path.display(), Error::description(&why)),
        Ok(file) => file,
    };
    
    let mut buf = Vec::<u8>::new();

    let result = match file.read_to_end(&mut buf) {
        Err(why) => panic!("Error reading file: {}", Error::description(&why)),
        Ok(result) => println!("Read {}: {} bytes", path.display(), result),
    };    

    buf
}
