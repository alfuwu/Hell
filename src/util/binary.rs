use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Error, Read, Write};

pub fn peek(buf: &mut BufReader<File>) -> Result<u8, Error> {
    let buffer = buf.fill_buf()?;
    Ok(buffer[0])
}
pub fn peek_many(buf: &mut BufReader<File>, amount: usize) -> Result<Vec<u8>, Error> {
    let buffer = buf.fill_buf()?;
    let len = buffer.len().min(amount);
    Ok(buffer[..len].to_vec())
}

pub fn read_byte(buf: &mut BufReader<File>) -> Result<u8, Error> {
    let out = &mut [0u8; 1];
    buf.read_exact(out)?;
    Ok(out[0])
}
pub fn read_u32(buf: &mut BufReader<File>) -> Result<u32, Error> {
    let out = &mut [0u8; 4];
    buf.read_exact(out)?;
    Ok(u32::from_le_bytes(*out))
}
pub fn read_i32(buf: &mut BufReader<File>) -> Result<i32, Error> {
    let out = &mut [0u8; 4];
    buf.read_exact(out)?;
    Ok(i32::from_le_bytes(*out))
}
pub fn read_f32(buf: &mut BufReader<File>) -> Result<f32, Error> {
    let out = &mut [0u8; 4];
    buf.read_exact(out)?;
    Ok(f32::from_le_bytes(*out))
}
pub fn read_string(buf: &mut BufReader<File>) -> Result<String, Error> {
    let size = read_u32(buf)? as usize;
    read_fixed_string(buf, size)
}
pub fn read_fixed_string(buf: &mut BufReader<File>, size: usize) -> Result<String, Error> {
    let mut string_bytes = vec![0u8; size];
    buf.read_exact(&mut string_bytes)?;
    Ok(String::from_utf8(string_bytes).unwrap()) // convert bytes to String
}

pub fn write_byte(writer: &mut BufWriter<File>, value: u8) -> Result<(), Error> {
    writer.write_all(&[value])
}
pub fn write_i32(writer: &mut BufWriter<File>, value: i32) -> Result<(), Error> {
    writer.write_all(&value.to_le_bytes())
}
pub fn write_u32(writer: &mut BufWriter<File>, value: u32) -> Result<(), Error> {
    writer.write_all(&value.to_le_bytes())
}
pub fn write_f32(writer: &mut BufWriter<File>, value: f32) -> Result<(), Error> {
    writer.write_all(&value.to_bits().to_le_bytes())
}
pub fn write_string(writer: &mut BufWriter<File>, value: &str) -> Result<(), Error> {
    write_u32(writer, value.len() as u32)?;
    writer.write_all(value.as_bytes())
}
pub fn write_fixed_string(writer: &mut BufWriter<File>, value: &str) -> Result<(), Error> {
    writer.write_all(value.as_bytes())
}