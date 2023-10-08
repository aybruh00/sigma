use std::net::SocketAddr::{V4, V6};
use tokio::net::{TcpSocket, TcpStream, lookup_host};
use tokio::io::copy;

pub struct HttpProxyTunnel {
    incoming_sock: TcpStream,
    incoming_remote_addr: String,
    // mut outgoingSock: Option<TcpStream>,
    outgoing_local_addr: String,
    buf: [u8; 4096]
}

impl HttpProxyTunnel {
    pub async fn start(&mut self) {
        // let mut client_remote_buf: [u8] = [0u8; 4096];
        self.read_from_client().await; // (&mut client_remote_buf);
    } 

    async fn connect_remote //<E: std::convert::From<std::io::Error>> 
    (&self, hostname: &str) -> Result<TcpStream, std::io::Error> {
        for host in lookup_host(hostname).await? {
            match host {
                V4(_) => {
                    let remote_sock = TcpSocket::new_v4()?;
                    remote_sock.bind((self.outgoing_local_addr.clone()+":0").parse().unwrap());
                    return remote_sock.connect(host).await;
                }
                V6(_) => {
                    let remote_sock = TcpSocket::new_v6()?;
                    remote_sock.bind((self.outgoing_local_addr.clone()+":0").parse().unwrap());
                    return remote_sock.connect(host).await;
                }
            }
        }
        Err(std::io::Error::new(std::io::ErrorKind::Other, "No hosts returned!"))
    }

    async fn read_from_client(&mut self) {
        loop{
            match self.incoming_sock.try_read(&mut (self.buf)) {
                Ok(0) => break,
                Ok(n) => {
                    self.process_request(n).await;
                }
                _ => break,
            }
        }
    }

    async fn process_request (&mut self, bytes: usize) {
        let (mut host_name_idx, mut endl_idx) = (0usize, 0usize);
        let mut host_found = false;
        let mut endl_found = false;

        for i in 0..(bytes-5) {
            if &self.buf[i..i+6] == b"Host: " {
                host_name_idx = i+6;
                host_found = true;
                break;
            }
        }

        if host_found{
            for i in host_name_idx..(bytes-1) {
                if &self.buf[i..i+2] == b"\r\n" {
                    endl_idx = i;
                    endl_found = true;
                }
            }
        }

        if endl_found {
            let hostname = std::str::from_utf8(&self.buf[host_name_idx..endl_idx]).unwrap();
            let mut remote_sock = self.connect_remote(hostname).await.unwrap();
            let mut bytes_written = 0;
            while bytes_written < bytes {
                bytes_written += remote_sock.try_write(&self.buf[bytes_written..bytes]).unwrap();
            }

            let (mut remote_read, mut client_write) = (remote_sock.split().0, self.incoming_sock.split().1);
            copy(&mut remote_read, &mut client_write).await;
        }
    }
}