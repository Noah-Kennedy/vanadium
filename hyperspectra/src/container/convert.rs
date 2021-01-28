use crate::bar::config_bar;
use crate::container::{IterableImage, IterableImageMut, LockImage, SizedImage};

pub fn convert<'a, I, O, T>(input: &LockImage<T, I>, output: &mut LockImage<T, O>)
    where I: IterableImage<'a, T> + SizedImage + 'static + Send + Sync,
          O: IterableImageMut<'a, T> + SizedImage + 'static + Send + Sync,
          T: Copy + 'static + Send + Sync
{
    rayon::scope(move |s| {
        let input = input.read().inner;
        let mut output = output.write().inner;
        assert_eq!(input.dims(), output.dims(), "Dims mismatch error, contact the developer!");

        let bar = indicatif::ProgressBar::new(input.dims().channels as u64);

        config_bar(&bar, "Converting bands");

        for (in_band, out_band) in input.bands().zip(output.bands_mut()) {
            let bar = bar.clone();
            s.spawn(move |_| {
                for (in_cell, out_cell) in in_band.zip(out_band) {
                    *out_cell = *in_cell
                }

                bar.inc(1);
            })
        };

        bar.finish();
    });
}