use futures::{select, FutureExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, DuplexStream};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
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
        let (mut r, mut w) = conn.into_split();
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
            loop {
                if let Err(err) = Self::read_pack(&mut r, &duplex_map).await {
                    todo!()
                }
            }
        });
    }
    async fn read_pack(conn: &mut OwnedReadHalf, duplex_map: &Mutex<HashMap<u16, DuplexStream>>) -> Result<(), std::io::Error> {
        let comp_port = conn.read_u16().await?;
        let pack_size = conn.read_u32().await?;

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
    fn poll_write(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, std::io::Error>> {
        todo!()
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        todo!()
    }

    fn poll_shutdown(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        todo!()
    }
}
