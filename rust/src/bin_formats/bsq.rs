use rayon::prelude::*;

use crate::bin_formats::THREAD_WORK_UNIT_SIZE;
use crate::headers::envi::EnviByteOrder;
use crate::prelude::EnviHeaders;

pub struct BorrowedBsqData<'a, T> {
    pub bands: Vec<&'a [T]>,
}

pub struct BorrowedBsqDataMut<'a, T> {
    pub bands: Vec<&'a mut [T]>,
}

impl<'a, T> BorrowedBsqData<'a, T> {
    pub fn new_from_raw((headers, raw): (&EnviHeaders, &'a [T])) -> Self {
        assert_eq!(EnviByteOrder::Intel, headers.byte_order);
        assert_eq!(0, headers.header_offset);
        assert_eq!(raw.len(), headers.bands * headers.samples * headers.lines);

        let chunk_size = raw.len() / headers.bands;

        let bands: Vec<&'a [T]> = raw.chunks(chunk_size).collect();

        Self {
            bands
        }
    }
}

impl<'a, T> Into<BorrowedBsqData<'a, T>> for BorrowedBsqDataMut<'a, T> {
    fn into(self) -> BorrowedBsqData<'a, T> {
        let bands: Vec<&[T]> = self.bands
            .into_iter()
            .map(|band| band as &[T])
            .collect();

        BorrowedBsqData { bands }
    }
}

impl<'a, T> BorrowedBsqDataMut<'a, T> {
    pub fn new_from_raw((headers, raw): (&EnviHeaders, &'a mut [T])) -> Self {
        assert_eq!(EnviByteOrder::Intel, headers.byte_order);
        assert_eq!(0, headers.header_offset);
        assert_eq!(raw.len(), headers.bands * headers.samples * headers.lines);

        let chunk_size = raw.len() / headers.bands;

        let bands: Vec<&'a mut [T]> = raw.chunks_mut(chunk_size).collect();

        Self {
            bands
        }
    }
}

impl<'a, T> BorrowedBsqDataMut<'a, T> where T: Copy + Send {
    #[inline(always)]
    pub fn map_in_place<F>(&mut self, fun: F) where F: Fn(T) -> T + Send + Sync + Copy {
        self.bands.iter_mut().for_each(|band|
            band.par_chunks_mut(THREAD_WORK_UNIT_SIZE)
                .for_each(|chunk|
                    if chunk.len() % 8 == 0 {
                        unsafe { Self::big_chungus(chunk, fun) }
                    } else {
                        Self::little_chungus(chunk, fun)
                    }))
    }

    /// Only works if chunk is divisible by 8
    #[inline(always)]
    unsafe fn big_chungus<F>(chunk: &mut [T], fun: F) where F: Fn(T) -> T + Send + Sync {
        for chungus in chunk.chunks_mut(8) {
            // unrolled in batches of size 8 to encourage the compiler to vectorize
            *chungus.get_unchecked_mut(0) = fun(*chungus.get_unchecked(0));
            *chungus.get_unchecked_mut(1) = fun(*chungus.get_unchecked(1));
            *chungus.get_unchecked_mut(2) = fun(*chungus.get_unchecked(2));
            *chungus.get_unchecked_mut(3) = fun(*chungus.get_unchecked(3));
            *chungus.get_unchecked_mut(4) = fun(*chungus.get_unchecked(4));
            *chungus.get_unchecked_mut(5) = fun(*chungus.get_unchecked(5));
            *chungus.get_unchecked_mut(6) = fun(*chungus.get_unchecked(6));
            *chungus.get_unchecked_mut(7) = fun(*chungus.get_unchecked(7));
        }
    }

    #[inline(always)]
    fn little_chungus<F>(chunk: &mut [T], fun: F) where F: Fn(T) -> T + Send + Sync {
        for item in chunk {
            *item = fun(*item)
        }
    }
}

impl<'a, T> BorrowedBsqData<'a, T>
    where T: Copy + Send + Sync
{
    #[inline(always)]
    pub fn map<F>(&self, out: &mut BorrowedBsqDataMut<'a, T>, fun: F)
        where F: Fn(T) -> T + Send + Sync
    {
        self.bands.iter()
            .zip(out.bands.iter_mut())
            .for_each(|(in_band, out_band)|
                in_band.chunks(THREAD_WORK_UNIT_SIZE)
                    .zip(out_band.chunks_mut(THREAD_WORK_UNIT_SIZE))
                    .par_bridge()
                    .for_each(|(in_chunk, out_chunk)|
                        in_chunk.iter()
                            .zip(out_chunk)
                            .for_each(|(input, output)| *output = fun(*input))))
    }
}