use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[derive(Clone)]
pub enum SocketAddr {
    TcpSocketAddr(std::net::SocketAddr),
    UnixSocketAddr(std::os::unix::net::SocketAddr),
}

impl Debug for SocketAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SocketAddr::TcpSocketAddr(addr) => write!(f, "http://{}", addr),
            SocketAddr::UnixSocketAddr(addr) => write!(f, "UnixSocketAddr({:?})", addr),
        }
    }
}

impl Display for SocketAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[allow(clippy::write_literal)]
            SocketAddr::TcpSocketAddr(addr) => write!(f, "{}", addr),
            SocketAddr::UnixSocketAddr(addr) => {
                write!(f, "{:?}", addr.as_pathname())
            }
        }
    }
}

impl From<std::net::SocketAddr> for SocketAddr {
    fn from(addr: std::net::SocketAddr) -> Self {
        SocketAddr::TcpSocketAddr(addr)
    }
}

impl From<std::os::unix::net::SocketAddr> for SocketAddr {
    fn from(addr: std::os::unix::net::SocketAddr) -> Self {
        SocketAddr::UnixSocketAddr(addr)
    }
}

impl FromStr for SocketAddr {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(addr) = s.parse::<std::net::SocketAddr>() {
            Ok(SocketAddr::TcpSocketAddr(addr))
        } else if let Ok(addr) = std::os::unix::net::SocketAddr::from_pathname(s) {
            Ok(SocketAddr::UnixSocketAddr(addr))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid socket address",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::socket_addr::SocketAddr;
    use std::path::Path;

    #[test]
    fn test_socket_addr() {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
        let socket_addr = SocketAddr::from(addr);
        assert_eq!(format!("{}", socket_addr), "127.0.0.1:8080");

        let _ = std::fs::remove_file("/tmp/sock");
        let addr = std::os::unix::net::SocketAddr::from_pathname("/tmp/sock").unwrap();
        let socket_addr = SocketAddr::from(addr);
        assert_eq!(
            format!("{}", socket_addr),
            format!("{:?}", Some(Path::new("/tmp/sock")))
        );
    }
}
