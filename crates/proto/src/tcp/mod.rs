/*
 * Copyright (C) 2015 Benjamin Fry <benjaminfry@me.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! TCP protocol related components for DNS
mod tcp_client_stream;
mod tcp_stream;

pub use self::tcp_client_stream::{TcpClientConnect, TcpClientStream};
pub use self::tcp_stream::{Connect, DnsTcpStream, TcpStream};

#[cfg(feature = "tokio-runtime")]
#[doc(hidden)]
pub mod tokio {
    use socket2::{Domain, Protocol, Socket, Type};
    use std::net::SocketAddr;
    use std::{io, net::IpAddr};
    use tokio::net::TcpStream as TokioTcpStream;
    use tokio::task::spawn_blocking;

    pub async fn connect(
        addr: &SocketAddr,
        bind_addr: &Option<IpAddr>,
    ) -> Result<TokioTcpStream, io::Error> {
        let stream = match bind_addr {
            Some(bind_addr) => {
                let addr = *addr;
                let bind_addr = *bind_addr;
                spawn_blocking(move || {
                    let domain = match bind_addr {
                        IpAddr::V4(_) => Domain::ipv4(),
                        IpAddr::V6(_) => Domain::ipv6(),
                    };
                    let socket = Socket::new(domain, Type::stream(), Some(Protocol::tcp()))?;
                    // Binding to port zero lets the OS assign a free port.
                    socket.bind(&SocketAddr::new(bind_addr, 0).into())?;
                    socket.connect(&addr.into())?;
                    let stream = TokioTcpStream::from_std(socket.into_tcp_stream())?;
                    Ok::<_, io::Error>(stream)
                })
                .await
                .unwrap()?
            }
            None => TokioTcpStream::connect(addr).await?,
        };
        stream.set_nodelay(true)?;
        Ok(stream)
    }
}
