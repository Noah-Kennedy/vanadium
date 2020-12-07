use std::ops::{Deref, DerefMut};

use indicatif::ProgressBar;

use crate::bin_formats::{FileDims, FileIndex, Mat};

impl<C1, I1> Mat<C1, f32, I1>
    where I1: 'static + FileIndex + Sync + Send + Copy + Clone,
          C1: Deref<Target=[u8]> + Sync + Send,
{
    pub fn convert<C2, I2>(&self, out: &mut Mat<C2, f32, I2>)
        where
            I2: 'static + FileIndex + Sync + Send + Copy + Clone,
            C2: DerefMut<Target=[u8]> + Sync + Send,
    {
        let FileDims { bands, samples, lines } = self.inner.size();
        let bands = bands.len();

        let bar = ProgressBar::new((lines * samples * bands) as u64);

        let r_idx_gen = self.index;
        let w_idx_gen = out.index;

        let r_ptr = unsafe { self.inner.get_unchecked() };
        let w_ptr = unsafe { out.inner.get_unchecked_mut() };

        for b in 0..bands {
            for l in 0..lines {
                for s in 0..samples {
                    let read_idx = r_idx_gen.get_idx(l, s, b);
                    let write_idx = w_idx_gen.get_idx(l, s, b);

                    unsafe {
                        let r = r_ptr.0.add(read_idx);
                        let w = w_ptr.0.add(write_idx);

                        w.write_volatile(r.read_volatile());
                    }
                }
            }

            bar.inc((lines * samples) as u64)
        }
    }
}