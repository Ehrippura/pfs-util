use std::{fs::File, io::{BufReader, Read, Seek, SeekFrom}};
use sha1::{Sha1, Digest};

use super::err::UnpackErr;

pub struct PFSHeader {
    pub info_size: u32,
    pub file_count: u32
}

impl PFSHeader {
    pub fn new() -> Self {
        Self {
            info_size: 0,
            file_count: 0
        }
    }
}

pub struct PFSEntityInfo {
    pub path: String,
    pub name: String,
    pub position: u32,
    pub offset: u32,
    pub size: u32
}

impl PFSEntityInfo {
    pub fn info_size(&self) -> u32 {

        let size = self.file_name_size();
        if size == 0 {
            return 0;
        }

        let u32_size = u32::try_from(size_of::<u32>()).unwrap();

        // offset size + file size + file name length size + file name length + terminate
        return u32_size * 4 + size;
    }

    pub fn file_name_size(&self) -> u32 {
        u32::try_from(self.name.as_bytes().len()).unwrap_or(0)
    }
}

pub struct PFSArchive {
    pub filename: String,
    pub files: Vec<PFSEntityInfo>,
    pub key: [u8; 20]
}

impl PFSArchive {

    pub const FILE_MAGIC: [u8; 2] = [0x70, 0x66];

    pub const FILE_VERSION: u8 = 0x38;

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
            let capacity = usize::try_from(read_u32(&mut reader)?).unwrap();
            let mut buffer = vec![0; capacity];
            reader.read_exact(&mut buffer).unwrap();
            let filename: String = String::from_utf8(buffer).unwrap().replace("\\", "/");
            let position = reader.stream_position().unwrap();
            let _skip: u32 = read_u32(&mut reader)?;
            let offset = read_u32(&mut reader)?;
            let size = read_u32(&mut reader)?;

            infos.push(PFSEntityInfo {
                path: String::from(""),
                name: filename,
                position: u32::try_from(position).unwrap_or(0),
                offset,
                size
            });
        }

        // get xor key
        let mut key = [0_u8; 20];

        if file_version == '8' {
            let info_size_block_length = u64::try_from(size_of::<u32>()).unwrap();
            // skip magic + version + info_size block
            reader.seek(SeekFrom::Start(2 + 1 + info_size_block_length)).unwrap();
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

fn read_u32<R: Read>(reader: &mut R) -> Result<u32, UnpackErr> {

    let mut buffer = [0_u8; 4];

    if let Err(e) = reader.read_exact(&mut buffer) {
        return Err(UnpackErr { message: format!("{}", e) });
    }

    Ok(u32::from_le_bytes(buffer))
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::path::Path;
    use crate::pfs::{pack, unpack};

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

    #[test]
    fn pack_unpack_single_file() {
        let input_file = "./sample/demo.ini";
        let output_pfs = "./sample/demo.pfs";
        let unpack_folder = "./sample/unpacked";

        pack::pack(input_file, output_pfs).unwrap();

        let archive = PFSArchive::from_file(output_pfs).unwrap();
        unpack::unpack(&archive, Some(unpack_folder), false).unwrap();

        let unpacked_file = Path::new(unpack_folder).join("demo.ini");
        assert!(unpacked_file.exists());

        let original_content = fs::read_to_string(input_file).unwrap();
        let unpacked_content = fs::read_to_string(unpacked_file).unwrap();
        assert_eq!(original_content, unpacked_content);

        fs::remove_file(output_pfs).unwrap();
        fs::remove_dir_all(unpack_folder).unwrap();
    }

    #[test]
    fn pack_unpack_folder() {
        let input_folder = "./sample/demo_folder";
        let output_pfs = "./sample/demo_folder.pfs";
        let unpack_folder = "./sample/unpacked_folder";

        pack::pack(input_folder, output_pfs).unwrap();

        let archive = PFSArchive::from_file(output_pfs).unwrap();
        unpack::unpack(&archive, Some(unpack_folder), false).unwrap();

        let inner_file = Path::new(unpack_folder).join("inner.ini");
        assert!(inner_file.exists());

        let original_content = fs::read_to_string(Path::new(input_folder).join("inner.ini")).unwrap();
        let unpacked_content = fs::read_to_string(inner_file).unwrap();
        assert_eq!(original_content, unpacked_content);

        let inner2_file = Path::new(unpack_folder).join("inner2.ini");
        assert!(inner2_file.exists());

        let original_content2 = fs::read_to_string(Path::new(input_folder).join("inner2.ini")).unwrap();
        let unpacked_content2 = fs::read_to_string(inner2_file).unwrap();
        assert_eq!(original_content2, unpacked_content2);

        fs::remove_file(output_pfs).unwrap();
        fs::remove_dir_all(unpack_folder).unwrap();
    }
}