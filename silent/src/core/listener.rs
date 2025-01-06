use super::socket_addr::SocketAddr;
use super::stream::Stream;

pub enum Listener {
    TcpListener(std::net::TcpListener),
    UnixListener(std::os::unix::net::UnixListener),
    TokioListener(tokio::net::TcpListener),
    TokioUnixListener(tokio::net::UnixListener),
}

impl From<std::net::TcpListener> for Listener {
    fn from(listener: std::net::TcpListener) -> Self {
        Listener::TcpListener(listener)
    }
}

impl From<std::os::unix::net::UnixListener> for Listener {
    fn from(value: std::os::unix::net::UnixListener) -> Self {
        Listener::UnixListener(value)
    }
}

impl From<tokio::net::TcpListener> for Listener {
    fn from(listener: tokio::net::TcpListener) -> Self {
        Listener::TokioListener(listener)
    }
}

impl From<tokio::net::UnixListener> for Listener {
    fn from(value: tokio::net::UnixListener) -> Self {
        Listener::TokioUnixListener(value)
    }
}

impl Listener {
    pub async fn accept(&self) -> std::io::Result<(Stream, SocketAddr)> {
        match self {
            Listener::TcpListener(listener) => {
                let (stream, addr) = listener.accept()?;
                Ok((
                    Stream::TcpStream(tokio::net::TcpStream::from_std(stream)?),
                    SocketAddr::TcpSocketAddr(addr),
                ))
            }
            Listener::UnixListener(listener) => {
                let (stream, addr) = listener.accept()?;
                Ok((
                    Stream::UnixStream(tokio::net::UnixStream::from_std(stream)?),
                    SocketAddr::UnixSocketAddr(addr),
                ))
            }
            Listener::TokioListener(listener) => {
                let (stream, addr) = listener.accept().await?;
                Ok((Stream::TcpStream(stream), SocketAddr::TcpSocketAddr(addr)))
            }
            Listener::TokioUnixListener(listener) => {
                let (stream, addr) = listener.accept().await?;
                Ok((
                    Stream::UnixStream(stream),
                    SocketAddr::UnixSocketAddr(addr.into()),
                ))
            }
        }
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        match self {
            Listener::TcpListener(listener) => listener.local_addr().map(SocketAddr::TcpSocketAddr),
            Listener::UnixListener(listener) => {
                Ok(SocketAddr::UnixSocketAddr(listener.local_addr()?))
            }
            Listener::TokioListener(listener) => {
                listener.local_addr().map(SocketAddr::TcpSocketAddr)
            }
            Listener::TokioUnixListener(listener) => {
                Ok(SocketAddr::UnixSocketAddr(listener.local_addr()?.into()))
            }
        }
    }
}
