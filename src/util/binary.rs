use std::io::{BufRead, BufReader, BufWriter, Error, Read, Write};
use crate::util::matrices::{Matrix3f, Matrix4f};
use crate::util::quaternion::Quaternionf;
use crate::util::vectors::Vector3f;

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
    let out = &mut [0; 1];
    buf.read_exact(out)?;
    Ok(out[0])
}
pub fn read_u16<R: Read>(buf: &mut BufReader<R>) -> Result<u16, Error> {
    let out = &mut [0; 2];
    buf.read_exact(out)?;
    Ok(u16::from_le_bytes(*out))
}
pub fn read_i16<R: Read>(buf: &mut BufReader<R>) -> Result<i16, Error> {
    let out = &mut [0; 2];
    buf.read_exact(out)?;
    Ok(i16::from_le_bytes(*out))
}
pub fn read_u32<R: Read>(buf: &mut BufReader<R>) -> Result<u32, Error> {
    let out = &mut [0; 4];
    buf.read_exact(out)?;
    Ok(u32::from_le_bytes(*out))
}
pub fn read_i32<R: Read>(buf: &mut BufReader<R>) -> Result<i32, Error> {
    let out = &mut [0; 4];
    buf.read_exact(out)?;
    Ok(i32::from_le_bytes(*out))
}
pub fn read_f32<R: Read>(buf: &mut BufReader<R>) -> Result<f32, Error> {
    let out = &mut [0; 4];
    buf.read_exact(out)?;
    Ok(f32::from_le_bytes(*out))
}
pub fn read_string<R: Read>(buf: &mut BufReader<R>) -> Result<String, Error> {
    let size = read_u32(buf)? as usize;
    read_fixed_string(buf, size)
}
pub fn read_fixed_string<R: Read>(buf: &mut BufReader<R>, size: usize) -> Result<String, Error> {
    let mut string_bytes = vec![0; size];
    buf.read_exact(&mut string_bytes)?;
    Ok(String::from_utf8(string_bytes).unwrap()) // convert bytes to String
}
pub fn read_vector3f<R: Read>(buf: &mut BufReader<R>) -> Result<Vector3f, Error> {
    Ok(Vector3f::new(read_f32(buf)?, read_f32(buf)?, read_f32(buf)?))
}
pub fn read_quaternionf<R: Read>(buf: &mut BufReader<R>) -> Result<Quaternionf, Error> {
    Ok(Quaternionf::new(read_f32(buf)?, read_f32(buf)?, read_f32(buf)?, read_f32(buf)?))
}
pub fn read_matrix3f<R: Read>(buf: &mut BufReader<R>) -> Result<Matrix3f, Error> {
    let mut m = [[0.0; 3]; 3];
    for i in 0..9 {
        m[i/3][i%3] = read_f32(buf)?;
    }
    Ok(Matrix3f::new(m))
}
pub fn read_matrix4f<R: Read>(buf: &mut BufReader<R>) -> Result<Matrix4f, Error> {
    let mut m = [[0.0; 4]; 4];
    for i in 0..16 {
        m[i/4][i%4] = read_f32(buf)?;
    }
    Ok(Matrix4f::new(m))
}

pub fn write_byte<W: Write>(writer: &mut BufWriter<W>, value: u8) -> Result<(), Error> {
    writer.write_all(&[value])
}
pub fn write_u16<W: Write>(writer: &mut BufWriter<W>, value: u16) -> Result<(), Error> {
    writer.write_all(&value.to_le_bytes())
}
pub fn write_i16<W: Write>(writer: &mut BufWriter<W>, value: i16) -> Result<(), Error> {
    writer.write_all(&value.to_le_bytes())
}
pub fn write_u32<W: Write>(writer: &mut BufWriter<W>, value: u32) -> Result<(), Error> {
    writer.write_all(&value.to_le_bytes())
}
pub fn write_i32<W: Write>(writer: &mut BufWriter<W>, value: i32) -> Result<(), Error> {
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
pub fn write_vector3f<W: Write>(writer: &mut BufWriter<W>, value: Vector3f) -> Result<(), Error> {
    write_f32(writer, value.x)?;
    write_f32(writer, value.y)?;
    write_f32(writer, value.z)?;
    Ok(())
}
pub fn write_quaternionf<W: Write>(writer: &mut BufWriter<W>, value: Quaternionf) -> Result<(), Error> {
    write_f32(writer, value.x)?;
    write_f32(writer, value.y)?;
    write_f32(writer, value.z)?;
    write_f32(writer, value.w)?;
    Ok(())
}
pub fn write_matrix3f<W: Write>(writer: &mut BufWriter<W>, value: Matrix3f) -> Result<(), Error> {
    for i in 0..9 {
        write_f32(writer, value.m[i/3][i%3])?;
    }
    Ok(())
}
pub fn write_matrix4f<W: Write>(writer: &mut BufWriter<W>, value: Matrix4f) -> Result<(), Error> {
    for i in 0..16 {
        write_f32(writer, value.m[i/4][i%4])?;
    }
    Ok(())
}