use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::fs::{self, File};

use super::archive::PFSArchive;
use super::err::UnpackErr;

pub fn unpack(archive: &PFSArchive, target: Option<&str>, dry: bool) -> Result<(), UnpackErr> {

    let target_folder = if let Some(name) = target {
        Path::new(name)
    } else {
        let name = Path::new(archive.filename.as_str())
            .file_stem()
            .expect("file stem not available")
            .to_str()
            .expect("cannot convert OSStr to str");

        Path::new(name)
    };

    println!("Start unarchive file: {}", archive.filename);

    println!("Save file to target: {}", target_folder.to_str().unwrap());

    if dry {
        for entity in &archive.files {
            println!("extract file: {}...OK", entity.name.as_str());
        }
        return Ok(())
    }

    if target_folder.exists() {
        if !target_folder.is_dir() {
            return Err(UnpackErr { message: format!("{} is not folder", target_folder.to_str().unwrap()) });
        }
    } else {
        fs::create_dir(target_folder).unwrap();
    }

    let file = File::options()
        .read(true)
        .write(false)
        .open(archive.filename.as_str())
        .unwrap();

    let mut reader = BufReader::new(file);

    for entity in &archive.files {
        let file_path = target_folder.join(entity.name.as_str());
        print!("extract file: {}...", entity.name.as_str());

        let prefix = file_path.parent().unwrap();
        if prefix.exists() {
            if !target_folder.is_dir() {
                return Err(UnpackErr { message: format!("{} is not folder", prefix.to_str().unwrap()) });
            }
        } else {
            fs::create_dir_all(prefix).unwrap();
        }

        let mut buffer = vec![0_u8; usize::try_from(entity.size).unwrap()];
        let mut final_data = vec![0_u8; usize::try_from(entity.size).unwrap()];

        let position = u64::from(entity.offset);

        reader.seek(SeekFrom::Start(position)).unwrap();
        reader.read_exact(&mut buffer).unwrap();

        let key_length = archive.key.len();

        for (idx, byte) in buffer.iter().enumerate() {
            final_data[idx] = byte ^ archive.key[idx % key_length];
        }

        match fs::write(file_path, final_data) {
            Ok(_) => print!("OK\n"),
            Err(e) => println!("\ncannot write data: {}", e)
        }
    }

    Ok(())
}
