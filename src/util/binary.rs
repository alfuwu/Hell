use std::fs::File;
use std::io::{BufRead, BufReader, Read};

pub fn read_f32(buf: &mut BufReader<File>) -> f32 {
    let out = &mut [0u8; 4];
    buf.read_exact(out).unwrap();
    f32::from_le_bytes(*out)
}

pub fn read_i32(buf: &mut BufReader<File>) -> i32 {
    let out = &mut [0u8; 4];
    buf.read_exact(out).unwrap();
    i32::from_le_bytes(*out)
}

pub fn read_string(buf: &mut BufReader<File>) -> String {
    let size = read_i32(buf) as usize;
    let mut string_bytes = vec![0u8; size];
    buf.read_exact(&mut string_bytes).unwrap();
    String::from_utf8(string_bytes).unwrap() // convert bytes to String
}

pub fn peek(buf: &mut BufReader<File>) -> u8 {
    let buffer = buf.fill_buf().unwrap();
    buffer[0]
}

pub fn peek_many(buf: &mut BufReader<File>, amount: usize) -> Vec<u8> {
    let buffer = buf.fill_buf().unwrap();
    let len = buffer.len().min(amount);
    buffer[..len].to_vec()
}