use flate2::read::MultiGzDecoder;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, prelude::*},
    path::Path,
};

const MAGIC_MAX_LEN: usize = 3;
const GZ_MAGIC: [u8; 3] = [0x1f, 0x8b, 0x08];
const BUFF_SIZE: usize = 512 * 1024;

fn magic_num<P: AsRef<Path> + Copy>(file_name: P) -> Result<[u8; MAGIC_MAX_LEN], Box<dyn Error>> {
    let mut buffer: [u8; MAGIC_MAX_LEN] = [0; MAGIC_MAX_LEN];
    let mut fp = File::open(file_name)?;
    let _ = fp.read(&mut buffer)?;
    Ok(buffer)
}

fn is_gzipped<P: AsRef<Path> + Copy>(file_name: P) -> Result<bool, Box<dyn Error>> {
    let buffer = magic_num(file_name)?;
    let gz_or_not =
        buffer[0] == GZ_MAGIC[0] && buffer[1] == GZ_MAGIC[1] && buffer[2] == GZ_MAGIC[2];
    Ok(gz_or_not
        || file_name
            .as_ref()
            .extension()
            .is_some_and(|ext| ext == "gz"))
}

pub fn read_file(file_name: String) -> Result<Box<dyn BufRead + Send>, Box<dyn Error>> {
    let gz_flag = is_gzipped(&file_name)?;
    let fp = File::open(&file_name)?;

    if gz_flag {
        let decoder = MultiGzDecoder::new(fp);
        Ok(Box::new(BufReader::with_capacity(BUFF_SIZE, decoder)))
    } else {
        Ok(Box::new(BufReader::with_capacity(BUFF_SIZE, fp)))
    }
}
