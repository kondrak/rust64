use std;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::str;
use std::fmt;
use c64::memory;

use byteorder::{BigEndian, ReadBytesExt};
use num::FromPrimitive;

#[derive(Debug)]
pub struct Crt {
    header: Header,
    chips: Vec<Chip>,
}

impl Crt {
    pub fn from_filename(filename: &str) -> Result<Crt, String> {
        let mut file = File::open(filename).map_err(|e| e.to_string())?;

        // Read Header
        let mut signature = [0u8; 16];
        file.read(&mut signature).map_err(|e| e.to_string())?;
        if &signature != b"C64 CARTRIDGE   " {
            return Err("Invalid cartridge signature".to_string())
        }
        let header_len = file.read_u32::<BigEndian>().map_err(|e| e.to_string())?;
        let mut version = [0u8;2];
        file.read(&mut version).map_err(|e| e.to_string())?;
        let hw_type = file.read_u16::<BigEndian>().map_err(|e| e.to_string())?;
        if hw_type != 0 {
            return Err("Unsupported cartridge type".to_string())
        }
        let exrom = file.read_u8().map_err(|e| e.to_string())?;
        let game = file.read_u8().map_err(|e| e.to_string())?;
        file.seek(SeekFrom::Start(0x20)).map_err(|e| e.to_string())?;
        let mut name = [0u8; 32];
        file.read(&mut name).map_err(|e| e.to_string())?;
        
        // Read Chips
        file.seek(SeekFrom::Start(header_len as u64)).map_err(|e| e.to_string())?;
        let mut chips: Vec<Chip> = Vec::new();
        let mut chip_signature = [0u8;4];
        loop {
            chip_signature = [0u8;4];
            file.read(&mut chip_signature).map_err(|e| e.to_string())?;
            if &chip_signature != b"CHIP" {
                break;
            }
            let length = file.read_u32::<BigEndian>().map_err(|e| e.to_string())?;
            let chip_type = ChipType::from_u16(file.read_u16::<BigEndian>()
                .map_err(|e| e.to_string())?).ok_or("Invalid chip type".to_string())?;
            let bank_number = file.read_u16::<BigEndian>().map_err(|e| e.to_string())?;
            let load_addr = file.read_u16::<BigEndian>().map_err(|e| e.to_string())?;
            let data_size = file.read_u16::<BigEndian>().map_err(|e| e.to_string())?;
            let mut data: Vec<u8> = vec![0u8; data_size as usize];
            file.read(&mut data).map_err(|e| e.to_string())?;

            chips.push(Chip {
                signature: chip_signature,
                length: length,
                chip_type: chip_type,
                bank_number: bank_number,
                load_addr: load_addr,
                data_size: data_size,
                data: data,
            });
        }


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

    pub fn load_into_memory(&self, mut memory: std::cell::RefMut::<memory::Memory>) {
        for chip in self.chips.iter() {
            let base_addr = chip.load_addr;
            for (offset, byte) in chip.data.iter().enumerate() {
                memory.write_byte(base_addr+offset as u16, *byte);
            }
        }
    }
}

struct Header {
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
    version: {:x}.{:02x}
    hw_type: {},
    exrom: {},
    game: {},
    name: {}
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

impl fmt::Debug for Chip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
"Chip {{
    signature: {},
    length: {} bytes,
    chip_type: {:?},
    bank_number: {},
    load_addr: 0x{:04x},
    data_size: {} bytes,
    data: (not shown)
}}",
            str::from_utf8(&self.signature).unwrap(),
            self.length,
            self.chip_type,
            self.bank_number,
            self.load_addr,
            self.data_size
        )
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    enum ChipType {
        ROM,
        RAM,
        Flash,
    }
}
