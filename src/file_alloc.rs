use std::fs::File;
use std::io;

pub fn allocate_file(file: &File, length: usize) -> io::Result<()> {
    file.set_len(length as u64)
}