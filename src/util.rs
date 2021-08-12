use std::ops::{Div, Sub};
use std::{slice, mem};

pub fn _standardize<T>(item: T, mean: T, std_dev: T) -> T where T: Copy + Sub<Output=T> + Div<Output=T> {
    (item - mean) / std_dev
}

pub(crate) unsafe fn make_raw_mut<T>(data: &mut [T]) -> &mut [u8] {
    let length = mem::size_of_val(data);
    let ptr = data.as_mut_ptr() as *mut u8;

    slice::from_raw_parts_mut(ptr, length)
}

pub(crate) unsafe fn make_raw<T>(data: &[T]) -> &[u8] {
    let length = mem::size_of_val(data);
    let ptr = data.as_ptr() as *mut u8;

    slice::from_raw_parts(ptr, length)
}