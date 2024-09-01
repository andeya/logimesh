#![allow(dead_code)]
use futures::{select, FutureExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, DuplexStream};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::warn;

pub struct StreamMux {
    /// The buf max size.
    /// Default: 1024.
    max_buf_size: usize,
    duplex_map: Arc<Mutex<HashMap<u16, DuplexStream>>>,
    write_half: Arc<Mutex<OwnedWriteHalf>>,
}

pub(crate) struct Stream {
    comp_port: u16,
    duplex: DuplexStream,
    write_half: Arc<Mutex<OwnedWriteHalf>>,
}

impl StreamMux {
    pub fn new(conn: TcpStream) -> Self {
        let (r, w) = conn.into_split();
        let duplex_map: Arc<Mutex<HashMap<u16, DuplexStream>>> = Default::default();
        Self::start_receiving(r, duplex_map.clone());
        Self {
            max_buf_size: 1024,
            duplex_map,
            write_half: Arc::new(Mutex::new(w)),
        }
    }
    /// Set the buf max size.
    /// Default: 1024.
    pub fn with_max_buf_size(mut self, max_buf_size: usize) -> Self {
        if max_buf_size <= 0 {
            self.max_buf_size = 1024;
        } else {
            self.max_buf_size = max_buf_size;
        }
        self
    }
    fn start_receiving(mut r: OwnedReadHalf, duplex_map: Arc<Mutex<HashMap<u16, DuplexStream>>>) {
        tokio::spawn(async move {
            let mut buf = Vec::new();
            loop {
                if let Err(err) = Self::read_pack(&mut r, &mut buf, &duplex_map).await {
                    warn!("[LOGIMESH] Error occurred when reading data from the connection: {}", err);
                }
            }
        });
    }
    async fn read_pack(conn: &mut OwnedReadHalf, buf: &mut Vec<u8>, duplex_map: &Mutex<HashMap<u16, DuplexStream>>) -> Result<(), std::io::Error> {
        let comp_port = conn.read_u16().await?;
        let pack_size = conn.read_u32().await?;
        let mut duplex_map = duplex_map.lock().await;
        let ds = duplex_map.get_mut(&comp_port).unwrap();
        buf.resize(pack_size as usize, 0);
        conn.read_exact(buf).await?;
        ds.write_all_buf(&mut buf.as_slice()).await?;
        ds.flush().await?;
        Ok(())
    }
    pub async fn new_stream(&self, comp_port: u16) -> Stream {
        let (duplex1, duplex2) = tokio::io::duplex(64);
        self.duplex_map.lock().await.insert(comp_port, duplex2);
        Stream {
            comp_port,
            duplex: duplex1,
            write_half: self.write_half.clone(),
        }
    }
}

impl AsyncRead for Stream {
    fn poll_read(self: std::pin::Pin<&mut Self>, _: &mut Context<'_>, buf: &mut tokio::io::ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        select! {
            r = self.get_mut().duplex.read_buf(buf).fuse() => Poll::Ready(r.map(|_|())),
            default => {
                Poll::Pending
            }
        }
    }
}

impl AsyncWrite for Stream {
    fn poll_write(self: std::pin::Pin<&mut Self>, _: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, std::io::Error>> {
        let f = async {
            let mut write_half = self.write_half.lock().await;
            write_half.write_u16(self.comp_port).await?;
            write_half.write_u32(buf.len() as u32).await?;
            write_half.write_all(buf).await?;
            Ok::<usize, std::io::Error>(2 + 4 + buf.len())
        };
        select! {
            r = f.fuse() => {
                Poll::Ready(r)
            },
            default => Poll::Pending,
        }
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        // TODO: Reduce the calling frequency
        select! {
            r = async{self.write_half.lock().await.flush().await}.fuse() => {
                Poll::Ready(r)
            },
            default => Poll::Pending,
        }
    }

    fn poll_shutdown(self: std::pin::Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        // There is no requirement for active closure.
        Poll::Ready(Ok(()))
    }
}
