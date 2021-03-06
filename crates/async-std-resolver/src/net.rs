// Copyright 2015-2020 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::net::SocketAddr;
use std::pin::Pin;
use std::{io, net::IpAddr};

use async_std::task::spawn_blocking;
use async_trait::async_trait;
use futures_io::{AsyncRead, AsyncWrite};
use socket2::{Domain, Protocol, Socket, Type};
use trust_dns_resolver::proto::tcp::{Connect, DnsTcpStream};
use trust_dns_resolver::proto::udp::UdpSocket;

use crate::time::AsyncStdTime;

pub struct AsyncStdUdpSocket(async_std::net::UdpSocket);

#[async_trait]
impl UdpSocket for AsyncStdUdpSocket {
    type Time = AsyncStdTime;

    async fn bind(addr: &SocketAddr) -> io::Result<Self> {
        async_std::net::UdpSocket::bind(addr)
            .await
            .map(AsyncStdUdpSocket)
    }

    async fn recv_from(&mut self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.recv_from(buf).await
    }

    async fn send_to(&mut self, buf: &[u8], target: &SocketAddr) -> io::Result<usize> {
        self.0.send_to(buf, target).await
    }
}

pub struct AsyncStdTcpStream(async_std::net::TcpStream);

impl DnsTcpStream for AsyncStdTcpStream {
    type Time = AsyncStdTime;
}

#[async_trait]
impl Connect for AsyncStdTcpStream {
    async fn connect(addr: SocketAddr, bind_addr: Option<IpAddr>) -> io::Result<Self> {
        let stream = match bind_addr {
            Some(bind_addr) => {
                spawn_blocking(move || {
                    let domain = match bind_addr {
                        IpAddr::V4(_) => Domain::ipv4(),
                        IpAddr::V6(_) => Domain::ipv6(),
                    };
                    let socket = Socket::new(domain, Type::stream(), Some(Protocol::tcp()))?;
                    // Binding to port zero lets the OS assign a free port.
                    socket.bind(&SocketAddr::new(bind_addr, 0).into())?;
                    socket.connect(&addr.into())?;
                    let stream = async_std::net::TcpStream::from(socket.into_tcp_stream());
                    Ok::<_, io::Error>(stream)
                })
                .await?
            }
            None => async_std::net::TcpStream::connect(addr).await?,
        };
        stream.set_nodelay(true)?;
        Ok(AsyncStdTcpStream(stream))
    }
}

impl AsyncWrite for AsyncStdTcpStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bytes: &[u8],
    ) -> std::task::Poll<std::result::Result<usize, std::io::Error>> {
        Pin::new(&mut self.0).poll_write(cx, bytes)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        Pin::new(&mut self.0).poll_close(cx)
    }
}

impl AsyncRead for AsyncStdTcpStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bytes: &mut [u8],
    ) -> std::task::Poll<std::result::Result<usize, std::io::Error>> {
        Pin::new(&mut self.0).poll_read(cx, bytes)
    }
}
