use std::fs::File;
use nix::fcntl::{fallocate, FallocateFlags};
use std::os::unix::io::AsRawFd;
use std::error::Error;

pub fn allocate_file(file: &File, length: usize) -> Result<(), Box<dyn Error>> {
    fallocate(file.as_raw_fd(),
              FallocateFlags::FALLOC_FL_ZERO_RANGE,
              0,
              length as i64
    )?;

    Ok(())
}