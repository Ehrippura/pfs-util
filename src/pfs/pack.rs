use std::fs::File;
use std::io::{BufWriter, ErrorKind, Read, Seek, Write};
use std::path::Path;
use sha1::{Sha1, Digest};
use super::archive::*;

pub fn pack<P: AsRef<Path>>(input: P, output: P) -> Result<(), std::io::Error> {

    let mut files: Vec<PFSEntityInfo> = vec![];

    if input.as_ref().is_dir() {
        todo!("implement folder iternator")
    } else {
        let entity = entity_from(input)?;
        files.push(entity);
    }

    let mut header = PFSHeader::new();
    let mut offset: usize = 0;

    for file in &files {
        header.info_size += file.info_size();
        header.file_count += 1;
    }

    let u32_size = u32::try_from(size_of::<u32>()).unwrap();

    // entity index size
    header.info_size += 1 * u32_size;
    // entity count
    header.info_size += 1 * u32_size;
    // offset table
    header.info_size += header.file_count * 2 * u32_size;
    // table EOF
    header.info_size += 2 * u32_size;
    // table
    header.info_size += 1 * u32_size;

    offset += usize::try_from(header.info_size).unwrap();

    let dest_file = File::create(output)?;

    let mut writer = BufWriter::new(dest_file);

    offset += writer.write(&PFSArchive::FILE_MAGIC)?;
    offset += writer.write(&[PFSArchive::FILE_VERSION])?;

    writer.write_all(&header.info_size.to_le_bytes())?;

    // reference position for offset table
    let ref_pos = writer.stream_position()?;

    let mut hasher = Sha1::new();

    offset += writer.write(&header.file_count.to_le_bytes())?;
    hasher.update(&header.file_count.to_le_bytes());

    // write file metadata
    for file in &mut files {
        let mut memory_writer = BufWriter::new(Vec::new());
        file.offset = u32::try_from(offset).unwrap();
        offset += memory_writer.write(&file.file_name_size().to_le_bytes())?;
        offset += memory_writer.write(file.name.as_bytes())?;
        file.position = u32::try_from(writer.stream_position()?).unwrap() + u32::try_from(memory_writer.buffer().len()).unwrap();
        offset += memory_writer.write(&[0_u8; 4])?;
        offset += memory_writer.write(&file.offset.to_le_bytes())?;
        offset += memory_writer.write(&file.size.to_le_bytes())?;
        offset += usize::try_from(file.size).unwrap();

        let bytes = memory_writer.into_inner()?;
        writer.write_all(bytes.as_ref())?;
        hasher.update(bytes);
    }

    // write offset table
    {
        let offset_table_position = writer.stream_position()?;
        let mut memory_writer = BufWriter::new(Vec::new());

        memory_writer.write_all(&(header.file_count + 1).to_le_bytes())?;

        for file in &files {
            memory_writer.write_all(&(file.position - u32::try_from(ref_pos).unwrap()).to_le_bytes())?;
            memory_writer.write_all(&[0_u8; 4])?;
        }

        // write EOF
        memory_writer.write_all(&[0_u8; 8])?;

        // write table position (position - magic - version)
        let table_pos = u32::try_from(offset_table_position - 3 - 4).unwrap();
        memory_writer.write_all(&table_pos.to_le_bytes())?;

        let bytes = memory_writer.into_inner()?;
        writer.write_all(bytes.as_ref())?;
        hasher.update(bytes);
    }

    let hash_key: [u8; 20] = hasher.finalize().into();

    for file in &files {
        let mut f = File::open(&file.path)?;
        let mut buffer = Vec::new();
        let mut final_data = Vec::new();
        let _read_size = f.read_to_end(&mut buffer)?;

        for (idx, byte) in buffer.iter().enumerate() {
            final_data.push(byte ^ hash_key[idx % hash_key.len()]);
        }

        writer.write_all(&final_data)?;
    }

    writer.flush().expect("failed to write file.");

    Ok(())
}

fn entity_from<P: AsRef<Path>>(path: P) -> Result<PFSEntityInfo, std::io::Error> {

    let path: &Path = path.as_ref();
    if !path.exists() {
        return Err(std::io::Error::from(ErrorKind::NotFound))
    }

    let file_size = std::fs::metadata(path).unwrap().len();
    let name = path.file_name().unwrap().to_str().unwrap();

    Ok(PFSEntityInfo {
        path: String::from(path.to_str().unwrap()),
        name: String::from(name),
        position: 0,
        offset: 0,
        size: u32::try_from(file_size).unwrap()
    })
}
