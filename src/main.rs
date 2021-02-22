extern crate byteorder;
extern crate serde;

use std::fs;
use std::io::prelude::*;
use std::mem;

use byteorder::{LittleEndian, ByteOrder, ReadBytesExt};
use serde::{Serialize, Deserialize};

static MAGIC: [u8;4] = [0x4c, 0x53, 0x50, 0x4b]; // LSPK

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PAKHeader {
    pub id: [u8; 4],
    pub version: u32,
    pub table_offset: usize,

}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PAK {
    pub header: PAKHeader,
    pub files: u32,
    pub table_zsize: u32,
    pub archives: Vec<PAKFile>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PAKFile {
    pub name: String,
    pub offset: u64,
    pub zsize: u64,
    pub size: u64,
    pub d1: u64,
    pub crc: u32,
    pub d2: u32,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("usage: bg3edit <target>");
        std::process::exit(1);
    }

    let target = &args[1];
    let data = match fs::read(target) {
        Ok(d) => d,
        Err(e) => panic!("error reading {}: {:?}", target, e)
    };

    if data.len() < mem::size_of::<PAKHeader>() {
        panic!("not enough data for header");
    }

    let mut id: [u8; 4] = [0, 0, 0, 0];
    id.copy_from_slice(&data[0..4]);
    let header = PAKHeader{
        id,
        version: LittleEndian::read_u32(&data[4..8]),
        table_offset: LittleEndian::read_u64(&data[8..16]) as usize
    };

    let holder = &data[header.table_offset..header.table_offset+8].to_vec();
    println!("{:02x?}", &holder);
    let files: u32 = LittleEndian::read_u32(&data[header.table_offset..header.table_offset+4]);
    let zsize: u32 = LittleEndian::read_u32(&data[header.table_offset+4..header.table_offset+8]);
    println!("HEADER:\n{:?}", header);
    println!("Files: {}, ZSIZE: {}", files, zsize);

    let mut rdr = std::io::Cursor::new(&data);
    let pak = read_pak(&mut rdr).expect("bad things");
    println!("PAK: {:?}", pak);
    println!("PAK: {:02x?}", pak);
}

fn read_pak<R: Read + Seek>(reader: &mut R) -> Result<PAK , std::io::Error> {
    let mut id = [0u8, 0, 0, 0];
    match reader.read(&mut id) {
        Ok(n) => if n != 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData, "header could not be read"))
            },
        Err(e) => return Err(e),
    };
    if id != MAGIC {
        panic!("magic")
    }

    let header = PAKHeader{
        id,
        version: reader.read_u32::<LittleEndian>()?,
        table_offset: reader.read_u64::<LittleEndian>()? as usize,
    };

    reader.seek(std::io::SeekFrom::Start(header.table_offset as u64))?;

    let pak = PAK{
        header,
        files: reader.read_u32::<LittleEndian>()?,
        table_zsize: reader.read_u32::<LittleEndian>()?,
    };
    Ok(pak)
}
