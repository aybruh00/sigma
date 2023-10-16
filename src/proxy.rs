use std::sync::Arc;
use std::net::SocketAddr::{self, V4, V6};
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::net::{TcpSocket, TcpStream, lookup_host};
use tokio::io::{self, copy};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use tokio_util::sync::CancellationToken as CT;
use tokio::time::timeout;


pub struct HttpProxyTunnel {
    pub outgoing_local_addr: SocketAddr,
    pub buf: Vec<u8>
}

impl HttpProxyTunnel {
    pub async fn start(&mut self, client_r: OwnedReadHalf, client_w: OwnedWriteHalf) -> io::Result<()> {
        self.read_from_client(client_r, Arc::new(Mutex::new(client_w))).await?; 
        Ok(())
    } 

    async fn read_from_client(&mut self, local_r: OwnedReadHalf, local_w: Arc<Mutex<OwnedWriteHalf>>) -> io::Result<()> {

        let (mut remote_r, mut remote_w): (Option<OwnedReadHalf>, Option<OwnedWriteHalf>) = (None, None);
        let mut token = CT::new();

        loop {
            if let Err(_e) = timeout(Duration::from_millis(10*100), local_r.readable()).await {
                // Timeout occured 
                // Cancel tasks
                token.cancel();
                println!("Exiting connection");
                return Ok(());
            }
            // local_r.readable().await?;
            match local_r.try_read(&mut self.buf) {
                Ok(0) => {
                    token.cancel();
                    return Ok(());
                }
                Ok(n) => {
                    token.cancel();

                    if let Some(s) = self.process_request(n).await {
                        let (a,b) = s.into_split();
                        remote_r = Some(a);
                        remote_w = Some(b);
                        token = CT::new();
                        
                        let cloned_token = token.clone();
                        let cloned_writer = local_w.clone();

                        tokio::spawn( async move {
                            HttpProxyTunnel::stream_remote_to_client(remote_r, cloned_writer, cloned_token).await;
                        });
                        io::copy_buf(&mut &self.buf[..n], remote_w.as_mut().unwrap()).await?;
                        remote_w.as_ref().unwrap().shutdown();
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(e) => {
                    token.cancel();
                    return Err(e);
                }
            };
        }
    }

    async fn stream_remote_to_client(mut remote_r: Option<OwnedReadHalf>, local_w: Arc<Mutex<OwnedWriteHalf>>, token: CT) {
        if remote_r.is_none() {return ();}
        let writer = &mut *(local_w.lock().await);

        loop {
            tokio::select!{
                _ = token.cancelled() => {
                    return ();
                    // finished task
                }
                _ = async {
                    let _ = copy(remote_r.as_mut().unwrap(), writer).await;
                }, if remote_r.is_some() => {}
            };
        }
    }

    async fn process_request (&self, bytes: usize) -> Option<TcpStream>{
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
            let hostname = String::from(std::str::from_utf8(&self.buf[host_name_idx..endl_idx]).unwrap());
            let sockaddr = self.outgoing_local_addr.clone();
            let handle: tokio::task::JoinHandle<io::Result<TcpStream>> = tokio::spawn(async move {
                Self::connect_remote(sockaddr, hostname).await
            });
            let conn_res = handle.await.unwrap();
            // let conn_res = self.connect_remote(hostname).await;
            if let Ok(_) = conn_res {
                return Some(conn_res.unwrap());
            }
        }
        None
    }

    async fn connect_remote (sockaddr: SocketAddr, hostname: String) -> io::Result<TcpStream> {
        for host in lookup_host(String::from(hostname)+":80").await? {
            match host {
                V4(_) => {
                    let remote_sock = TcpSocket::new_v4()?;
                    remote_sock.bind(sockaddr)?;
                    return remote_sock.connect(host).await;
                }
                V6(_) => {
                    continue;
                    // let remote_sock = TcpSocket::new_v6()?;
                    // remote_sock.bind(self.outgoing_local_addr);
                    // return remote_sock.connect(host).await;
                }
            }
        }
        Err(std::io::Error::new(std::io::ErrorKind::Other, "No hosts returned!"))
    }
}