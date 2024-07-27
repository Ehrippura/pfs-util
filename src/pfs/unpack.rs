use std::path::Path;

use super::archive::PFSArchive;
use super::err::UnpackErr;


pub fn unpack(archive: &PFSArchive, target: Option<&str>) -> Result<(), UnpackErr> {

    let target_folder = if let Some(name) = target {
        Path::new(name)
    } else {
        let name = Path::new(archive.filename.as_str())
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();

        Path::new(name)
    };

    println!("Start unarchive file: {}", archive.filename);

    println!("Save file to target: {}", target_folder.to_str().unwrap());

    println!("File list:");

    for file in &archive.files {
        println!("{}", file.name);
    }

    Ok(())
}
