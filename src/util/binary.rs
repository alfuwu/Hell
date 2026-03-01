use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Error, Read, Write};

pub fn peek<R: Read>(buf: &mut BufReader<R>) -> Result<u8, Error> {
    let buffer = buf.fill_buf()?;
    Ok(buffer[0])
}
pub fn peek_many<R: Read>(buf: &mut BufReader<R>, amount: usize) -> Result<Vec<u8>, Error> {
    let buffer = buf.fill_buf()?;
    let len = buffer.len().min(amount);
    Ok(buffer[..len].to_vec())
}

pub fn read_byte<R: Read>(buf: &mut BufReader<R>) -> Result<u8, Error> {
    let out = &mut [0u8; 1];
    buf.read_exact(out)?;
    Ok(out[0])
}
pub fn read_u32<R: Read>(buf: &mut BufReader<R>) -> Result<u32, Error> {
    let out = &mut [0u8; 4];
    buf.read_exact(out)?;
    Ok(u32::from_le_bytes(*out))
}
pub fn read_i32<R: Read>(buf: &mut BufReader<R>) -> Result<i32, Error> {
    let out = &mut [0u8; 4];
    buf.read_exact(out)?;
    Ok(i32::from_le_bytes(*out))
}
pub fn read_f32<R: Read>(buf: &mut BufReader<R>) -> Result<f32, Error> {
    let out = &mut [0u8; 4];
    buf.read_exact(out)?;
    Ok(f32::from_le_bytes(*out))
}
pub fn read_string<R: Read>(buf: &mut BufReader<R>) -> Result<String, Error> {
    let size = read_u32(buf)? as usize;
    read_fixed_string(buf, size)
}
pub fn read_fixed_string<R: Read>(buf: &mut BufReader<R>, size: usize) -> Result<String, Error> {
    let mut string_bytes = vec![0u8; size];
    buf.read_exact(&mut string_bytes)?;
    Ok(String::from_utf8(string_bytes).unwrap()) // convert bytes to String
}

pub fn write_byte<W: Write>(writer: &mut BufWriter<W>, value: u8) -> Result<(), Error> {
    writer.write_all(&[value])
}
pub fn write_i32<W: Write>(writer: &mut BufWriter<W>, value: i32) -> Result<(), Error> {
    writer.write_all(&value.to_le_bytes())
}
pub fn write_u32<W: Write>(writer: &mut BufWriter<W>, value: u32) -> Result<(), Error> {
    writer.write_all(&value.to_le_bytes())
}
pub fn write_f32<W: Write>(writer: &mut BufWriter<W>, value: f32) -> Result<(), Error> {
    writer.write_all(&value.to_bits().to_le_bytes())
}
pub fn write_string<W: Write>(writer: &mut BufWriter<W>, value: &str) -> Result<(), Error> {
    write_u32(writer, value.len() as u32)?;
    writer.write_all(value.as_bytes())
}
pub fn write_fixed_string<W: Write>(writer: &mut BufWriter<W>, value: &str) -> Result<(), Error> {
    writer.write_all(value.as_bytes())
}
