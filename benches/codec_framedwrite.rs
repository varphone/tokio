use bytes::{BufMut, BytesMut};
use futures::sink::SinkExt;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::AsyncWrite;
use tokio::runtime::{self, Runtime};
use tokio_util::codec::{Encoder, FramedWrite};

use bencher::{benchmark_group, benchmark_main, Bencher};

struct DummyWriter;

impl std::io::Write for DummyWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl AsyncWrite for DummyWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        src: &[u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(Ok(src.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
}

struct MyEncoder;

impl Encoder<u64> for MyEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: u64, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_u64(item);
        Ok(())
    }
}

fn rt() -> Runtime {
    runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
}

fn yield_latency(b: &mut Bencher) {
    let rt = rt();
    let mut sink = FramedWrite::new(DummyWriter {}, MyEncoder {});
    b.iter(|| {
        let _r = rt.block_on(async {
            let _r = sink.send(1234u64).await;
        });
    });
}

benchmark_group!(scheduler, yield_latency);

benchmark_main!(scheduler);
