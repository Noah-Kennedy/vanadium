use std::ops::{Div, Sub};

pub fn _standardize<T>(item: T, mean: T, std_dev: T) -> T where T: Copy + Sub<Output=T> + Div<Output=T> {
    (item - mean) / std_dev
}
