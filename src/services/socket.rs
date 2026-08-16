use anyhow::Result;
use serde::Serialize;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

pub struct SocketService {
    socket: UdpSocket,
}

impl SocketService {
    pub fn bind(addr: impl ToSocketAddrs) -> Result<Self> {
        Ok(SocketService {
            socket: UdpSocket::bind(addr)?,
        })
    }

    pub fn send_to<T: Serialize>(
        &self,
        data: &T,
        addr: SocketAddr,
        buf: &mut Vec<u8>,
    ) -> Result<usize> {
        buf.clear();
        bincode::serialize_into(&mut *buf, data)?;
        Ok(self.socket.send_to(buf, addr)?)
    }
}
