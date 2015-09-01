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
    
    let mut file_data = Vec::<u8>::new();

    let result = file.read_to_end(&mut file_data);

    match result {
        Err(why) => panic!("Error reading file: {}", Error::description(&why)),
        Ok(result) => println!("Read {}: {} bytes", path.display(), result),
    };    

    file_data
}
