use std::mem;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_uring::fs;

use crate::headers::Header;
use crate::specialization::bip::Bip;

const BACKPRESSURE_LIMIT: usize = 8;

pub struct TokioUringBip<T> {
    file: fs::File,
    bip: Bip<T>,
}

impl TokioUringBip<f32> {
    pub async fn new(headers: &Header) -> std::io::Result<Self> {
        let file = fs::OpenOptions::new()
            .read(true)
            .create(true)
            .open(&headers.path).await?;

        let bip = Bip {
            dims: headers.dims.clone(),
            phantom: Default::default(),
        };

        Ok(Self { file, bip })
    }

    pub async fn inline_means(&mut self) -> std::io::Result<Vec<f32>> {
        let (read_tx, mut read_rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) =
            tokio::sync::mpsc::channel(BACKPRESSURE_LIMIT);
        let (return_tx, return_rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) =
            tokio::sync::mpsc::channel(BACKPRESSURE_LIMIT);

        for _ in 0..BACKPRESSURE_LIMIT {
            return_tx.send(vec![0.0; self.bip.pixel_length()]).await.unwrap();
        }

        let bip = self.bip.clone();

        let handle = tokio_uring::spawn(async move {
            let mut accumulator = vec![0.0; bip.pixel_length()];

            while let Some(buffer) = read_rx.recv().await {
                bip.map_mean(&buffer, &mut accumulator);
                return_tx.send(buffer).await.unwrap();
            }

            accumulator
        });

        self.read_file(read_tx, return_rx).await?;

        let mut accumulator = handle.await.unwrap();

        self.bip.reduce_mean(&mut accumulator);

        Ok(accumulator)
    }

    pub async fn inline_std_devs(&mut self, means: Vec<f32>) -> std::io::Result<Vec<f32>> {
        let (read_tx, mut read_rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) =
            tokio::sync::mpsc::channel(BACKPRESSURE_LIMIT);
        let (return_tx, return_rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) =
            tokio::sync::mpsc::channel(BACKPRESSURE_LIMIT);

        for _ in 0..BACKPRESSURE_LIMIT {
            return_tx.send(vec![0.0; self.bip.pixel_length()]).await.unwrap();
        }

        let bip = self.bip.clone();

        let handle = tokio_uring::spawn(async move {
            let mut accumulator = vec![0.0; bip.pixel_length()];

            while let Some(buffer) = read_rx.recv().await {
                bip.map_std_dev(&buffer, &means, &mut accumulator);
                return_tx.send(buffer).await.unwrap();
            }

            accumulator
        });

        self.read_file(read_tx, return_rx).await?;

        let mut accumulator = handle.await.unwrap();

        self.bip.reduce_std_dev(&mut accumulator);

        Ok(accumulator)
    }

    pub async fn read_file(
        &mut self,
        read_tx: Sender<Vec<f32>>,
        mut return_rx: Receiver<Vec<f32>>,
    ) -> std::io::Result<()>
    {
        let pixel_length = self.bip.pixel_length() * mem::size_of::<f32>();

        for i in 0..self.bip.num_pixels() {
            let buf = return_rx.recv().await.unwrap();

            let raw_buf: Vec<u8> = unsafe {
                let mut raw_buf: Vec<u8> = mem::transmute(buf);

                raw_buf.set_len(mem::size_of::<f32>() * raw_buf.len());

                raw_buf
            };

            let (r, raw_read_buf) = self.file
                .read_at(raw_buf, i as u64 * pixel_length as u64).await;

            assert_eq!(r?, raw_read_buf.len());

            let float_buf: Vec<f32> = unsafe {
                let mut float_buf: Vec<f32> = mem::transmute(raw_read_buf);

                float_buf.set_len(float_buf.len() / mem::size_of::<f32>());

                float_buf
            };

            read_tx.send(float_buf).await.unwrap();
        }

        Ok(())
    }
}