use std::{fs::File, io::{BufRead, BufReader, Read, Seek, SeekFrom}, mem};
use sha1::{Sha1, Digest};

use super::err::UnpackErr;


pub struct PFSEntityInfo {
    pub name: String,
    pub offset: u32,
    pub size: u32
}

pub struct PFSArchive {
    pub filename: String,
    pub files: Vec<PFSEntityInfo>,
    pub key: [u8; 20]
}

impl PFSArchive {

    const FILE_MAGIC: [u8; 2] = [0x70, 0x66];

    const FILE_VERSION: u8 = 8;

    pub fn new(filename: &str) -> Self {
        Self {
            filename: String::from(filename),
            files: Vec::new(),
            key: [0_u8; 20]
        }
    }

    pub fn from_file(filename: &str) -> Result<Self, UnpackErr> {

        let file = File::options()
            .read(true)
            .write(false)
            .open(filename)
            .unwrap();

        let mut reader = BufReader::new(file);
        let mut buffer = [0; 2];

        if let Err(e) = reader.read_exact(&mut buffer) {
            return Err(UnpackErr { message: format!("{}", e) });
        }

        if Self::FILE_MAGIC != buffer {
            return Err(UnpackErr { message: String::from("File format not recognized") });
        }

        if let Err(e) = reader.read(&mut buffer[..1]) {
            return Err(UnpackErr { message: format!("{}", e) });
        }

        let file_version = char::from(buffer[0]);

        match file_version {
            '2' => println!("vaild PFS version 2"),
            '6' => println!("vaild PFS version 6"),
            '8' => println!("vaild PFS version 8"),
            _ => return Err(UnpackErr { message: String::from("Invalid file version") })
        }

        let info_size = read_u32(&mut reader)?;
        let file_count = read_u32(&mut reader)?;
        println!("File count {}", file_count);

        let mut infos: Vec<PFSEntityInfo> = vec![];

        for _ in 0..file_count {
            let path_length = read_u32(&mut reader)?;
            let capacity = usize::try_from(path_length).unwrap();
            let mut buffer = vec![0; capacity];
            reader.read_exact(&mut buffer).unwrap();
            let filename: String = String::from_utf8(buffer).unwrap();
            let _skip = read_u32(&mut reader)?;
            let offset = read_u32(&mut reader)?;
            let size = read_u32(&mut reader)?;

            infos.push(PFSEntityInfo {
                name: filename,
                offset,
                size
            });
        }

        // get xor key
        let mut key = [0_u8; 20];

        if file_version == '8' {
            reader.seek(SeekFrom::Start(2 + 1 + 4)).unwrap();
            let info_size_capacity = usize::try_from(info_size).unwrap();
            let mut buffer = vec![0; info_size_capacity];
            reader.read_exact(&mut buffer).unwrap();

            let mut hasher = Sha1::new();
            hasher.update(buffer);

            let result = hasher.finalize();
            key = result.into();
        }

        Ok(Self {
            filename: String::from(filename),
            files: infos,
            key
        })
    }
}

fn read_u32<R: BufRead>(reader: &mut R) -> Result<u32, UnpackErr> {

    let mut buffer = vec![0_u8; 4];

    if let Err(e) = reader.read_exact(&mut buffer) {
        return Err(UnpackErr { message: format!("{}", e) });
    }

    Ok(u32::from_le_bytes(buffer.try_into().unwrap()))
}

#[cfg(test)]
mod Test {

    use super::*;

    #[test]
    fn create_archive() {
        let archive = PFSArchive::new("hello.pfs");
        assert_eq!(archive.filename, "hello.pfs");
    }

    #[test]
    fn read_from_file() {
        let archive = PFSArchive::from_file("./sample/demo.pfs").unwrap();
        assert_eq!(archive.files.first().unwrap().name, "demo.ini");
    }
}