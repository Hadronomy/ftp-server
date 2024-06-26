use std::{
    borrow::BorrowMut,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use miette::*;
use num_integer::Integer;
use tokio::{
    io::AsyncWriteExt,
    net::{tcp::WriteHalf, TcpListener},
    sync::Mutex,
};
use tracing::*;

use crate::{DataConnection, FTPCommand, InnerConnectionRef, StatusCode};

pub struct Pasv;

impl<'a> FTPCommand<'a> for Pasv {
    const KEYWORD: &'static str = "PASV";

    async fn run<'b>(
        &self,
        connection: InnerConnectionRef,
        writer: &mut WriteHalf<'b>,
    ) -> Result<Option<StatusCode>> {
        // let ip_address = match local_ip().into_diagnostic()? {
        //     IpAddr::V4(ip) => ip,
        //     _ => return Err(miette!("Only IPv4 is supported")),
        // };
        let ip_address = Ipv4Addr::from([127, 0, 0, 1]);

        let data_addr = SocketAddr::from((ip_address, 0));
        let data_listener = TcpListener::bind(data_addr)
            .await
            .unwrap_or_else(|_| panic!("Could not bind to address {}", data_addr));
        let local_addr = data_listener.local_addr().into_diagnostic()?;
        let data_port = local_addr.port();
        let (port_high, port_low) = data_port.div_rem(&256);
        trace!("Data connection listener bound to {}", local_addr);

        writer
            .write(
                StatusCode::EnteringPassiveMode {
                    ip_address,
                    port_high,
                    port_low,
                }
                .to_string()
                .as_bytes(),
            )
            .await
            .into_diagnostic()?;

        writer.flush().await.into_diagnostic()?;

        trace!("Waiting for data connection");

        connection.lock().await.data_connection = None;
        let connection = connection.clone();
        tokio::spawn(async move {
            let connection_mutex = connection.lock();
            let (data_socket, _) = data_listener
                .accept()
                .await
                .expect("Error accepting connection to data_socket");

            trace!(
                "Data connection accepted from {}",
                data_socket.peer_addr().unwrap()
            );
            let data_connection = Arc::new(Mutex::new(DataConnection::from(data_socket)));
            connection_mutex.await.borrow_mut().data_connection = Some(data_connection);
            trace!("Data connection established");
        });

        Ok(None)
    }
}

impl<'a> TryFrom<(&'a str, Vec<&'a str>)> for Pasv {
    type Error = miette::Error;

    fn try_from((command, args): (&'a str, Vec<&'a str>)) -> Result<Self> {
        if command == Self::KEYWORD {
            if args.is_empty() {
                Ok(Self)
            } else {
                Err(miette!("Invalid number of arguments"))
            }
        } else {
            Err(miette!("Invalid command"))
        }
    }
}
