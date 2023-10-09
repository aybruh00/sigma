use std::net::SocketAddr::{V4, V6};
use tokio::net::{TcpSocket, TcpStream, lookup_host};
use tokio::io::copy;
// use tokio::time::Instant;

pub struct HttpProxyTunnel {
    pub incoming_sock: TcpStream,
    pub incoming_remote_addr: String,
    pub outgoing_local_addr: String,
    pub buf: Box<[u8; 4096]>
}

impl HttpProxyTunnel {
    pub async fn start(&mut self) {
        self.read_from_client().await; 
    } 

    async fn connect_remote(&self, hostname: &str) -> Result<TcpStream, std::io::Error> {
        println!("{hostname}");
        for host in lookup_host(String::from(hostname)+":80").await? {
            println!("{:?}", host);
            match host {
                V4(_) => {
                    let remote_sock = TcpSocket::new_v4()?;
                    remote_sock.bind((self.outgoing_local_addr.clone()+":0").parse().unwrap());
                    return remote_sock.connect(host).await;
                }
                V6(_) => {
                    continue;
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
            match self.incoming_sock.try_read(&mut (*self.buf)) {
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
                    break;
                }
            }
        }

        if endl_found {
            let hostname = std::str::from_utf8(&self.buf[host_name_idx..endl_idx]).unwrap();
            let mut remote_sock = self.connect_remote(hostname).await.unwrap();
            let mut bytes_written = 0;
            let (mut remote_read, mut remote_write) = remote_sock.split();
            let (mut client_read, mut client_write) = self.incoming_sock.split();
            while bytes_written < bytes {
                bytes_written += remote_write.try_write(&self.buf[bytes_written..bytes]).unwrap();
            }

            // let now = Instant::now();
            copy(&mut remote_read, &mut client_write).await;
            // let etime = now.elapsed();
            // println!("copy took {} seconds", etime.as_secs());
        }
    }
}