use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::str;

use std::fmt;

use byteorder::{BigEndian, ReadBytesExt};

pub struct Crt {
    pub header: Header,
    chips: Vec<Chip>,
}

impl Crt {
    pub fn load_from_file(filename: &str) -> Result<Crt, String> {
        let mut file = File::open(filename).map_err(|e| e.to_string())?;
        let mut signature = [0u8; 16];
        file.read(&mut signature).map_err(|e| e.to_string())?;
        if &signature != b"C64 CARTRIDGE   " {
            return Err("Invalid signature".to_string())
        }
        let header_len = file.read_u32::<BigEndian>().map_err(|e| e.to_string())?;
        let mut version = [0u8;2];
        file.read(&mut version).map_err(|e| e.to_string())?;
        let hw_type = file.read_u16::<BigEndian>().map_err(|e| e.to_string())?;
        let exrom = file.read_u8().map_err(|e| e.to_string())?;
        let game = file.read_u8().map_err(|e| e.to_string())?;
        file.seek(SeekFrom::Start(0x20)).map_err(|e| e.to_string())?;
        let mut name = [0u8; 32];
        file.read(&mut name).map_err(|e| e.to_string())?;
        let mut chips: Vec<Chip> = Vec::new();

        Ok(Crt {
            header: Header {
                signature: signature,
                header_len: header_len,
                version: version,
                hw_type: hw_type,
                exrom: exrom,
                game: game,
                name: name,
            },
            chips: chips,
        })
    }
}

pub struct Header {
    signature: [u8; 16],
    header_len: u32,
    version: [u8; 2],
    hw_type: u16,
    exrom: u8,
    game: u8,
    // 001A-001F RFU
    name: [u8; 32],
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, 
"Header {{
    signature: {},
    header_len: {} bytes,
    version: {:x}.{:x}
    hw_type: {},
    exrom: {},
    game: {},
    name: {},
}}",
            str::from_utf8(&self.signature).unwrap(),
            self.header_len,
            self.version[0],
            self.version[1],
            self.hw_type,
            self.exrom,
            self.game,
            str::from_utf8(&self.name).unwrap()
        )
    }
}

struct Chip {
    signature: [u8; 4],
    length: u32, // header and data combined
    chip_type: ChipType,
    bank_number: u16,
    load_addr: u16,
    data_size: u16,
    data: Vec<u8>, 
}

enum ChipType {
    ROM,
    RAM,
    Flash,
}
