// #![feature(cell_leak)]
// use std::cell::RefCell;
// use std::sync::RwLock;
use std::sync::Arc;
use std::net::SocketAddr::{V4, V6};
use tokio::sync::Mutex;
use tokio::net::{TcpSocket, TcpStream, lookup_host};
use tokio::io::{self, copy};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio_util::sync::CancellationToken as CT;

pub struct HttpProxyTunnel {
    // pub local_r: OwnedReadHalf,
    // pub local_w: OwnedWriteHalf,
    pub outgoing_local_addr: String,
    pub buf: Vec<u8>
}

impl HttpProxyTunnel {
    pub async fn start(&mut self, client_r: OwnedReadHalf, client_w: OwnedWriteHalf) {
        self.read_from_client(client_r, Arc::new(Mutex::new(client_w))).await; 
    } 

    async fn read_from_client(&mut self, local_r: OwnedReadHalf, local_w: Arc<Mutex<OwnedWriteHalf>>) -> io::Result<()> {

        let (mut remote_r, mut remote_w): (Option<OwnedReadHalf>, Option<OwnedWriteHalf>) = (None, None);
        let mut token = CT::new();

        loop {
            _ = local_r.readable().await?;
            match local_r.try_read(&mut self.buf) {
                Ok(0) => return Ok(()),
                Ok(n) => {
                    token.cancel();
                    token = CT::new();

                    if let Some(s) = self.process_request(n).await {
                        let (a,b) = s.into_split();
                        remote_r = Some(a);
                        remote_w = Some(b);
                        
                        let cloned_token = token.clone();
                        let cloned_writer = local_w.clone();

                        tokio::spawn( async move {
                            HttpProxyTunnel::stream_remote_to_client(remote_r, cloned_writer, cloned_token).await;
                        });

                        // loop {
                        //     match local_w.try_borrow_mut() {
                        //         Err(_) => continue,
                        //         Ok(writer) => {
                        //             tokio::spawn( async move {
                        //                 HttpProxyTunnel::stream_remote_to_client(remote_r, writer, cloned_token).await;
                        //             });
                        //             break;
                        //         }
                        //     }
                        // }

                        io::copy_buf(&mut &self.buf[..n], remote_w.as_mut().unwrap()).await?;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(e) => return Err(e),
            };
            

            /*
            tokio::select! {
                _ = self.local_r.readable()  => {
                    match self.local_r.try_read(&mut (*self.buf)) {
                        Ok(0) => {
                            println!("Closed"); 
                            return Ok(());
                        }
                        Ok(n) => {
                            if let Some(s) = self.process_request(n).await {
                                let (a, b) = s.into_split();
                                remote_r = Some(a);
                                remote_w = Some(b);
                            }
                            io::copy_buf(&mut &self.buf[..n], remote_w.as_mut().unwrap()).await;
                            // let mut bytes_written = 0;
                            // while bytes_written < n {
                            //     remote_w.as_ref().unwrap().writable().await;
                            //     if let Ok(t) = remote_w
                            //                         .as_mut()
                            //                         .unwrap()
                            //                         .try_write(&self.buf[bytes_written..n]) 
                            //     {
                            //         bytes_written += t;
                            //     }
                            // }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(e) => {
                            println!("{}", e);
                            return Err(e);
                        }
                    }
                }

                _ = async {
                    copy(remote_r.as_mut().unwrap(), &mut self.local_w).await;
                }, if remote_r.is_some() => {}
            };
            */
        }
        Ok(())
    }

    async fn stream_remote_to_client(mut remote_r: Option<OwnedReadHalf>, mut local_w: Arc<Mutex<OwnedWriteHalf>>, token: CT) {
        // if !remote_r.is_some() {return ();}
        // let writer = &mut *(local_w.write().unwrap());
        // tokio::pin!(writer);
        let writer = &mut *(local_w.lock().await);

        loop {
            tokio::select!{
                _ = token.cancelled() => {
                    return ();
                    // finished task
                }
                _ = async {
                    // let mutarc = Arc::get_mut(&mut local_w);
                    // if mutarc.is_some() {
                    //     copy(remote_r.as_mut().unwrap(), mutarc.unwrap()).await;
                    // }

                    copy(remote_r.as_mut().unwrap(), writer).await;
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
            let hostname = std::str::from_utf8(&self.buf[host_name_idx..endl_idx]).unwrap();
            let conn_res = self.connect_remote(hostname).await;
            if let Ok(_) = conn_res {
                return Some(conn_res.unwrap());
            }
        }
        None
    }

    async fn connect_remote (&self, hostname: &str) -> tokio::io::Result<TcpStream> {
        for host in lookup_host(String::from(hostname)+":80").await? {
            match host {
                V4(_) => {
                    let remote_sock = TcpSocket::new_v4()?;
                    remote_sock.bind((self.outgoing_local_addr.clone()+":0").parse().unwrap());
                    return remote_sock.connect(host).await;
                }
                V6(_) => {
                    continue;
                    // let remote_sock = TcpSocket::new_v6()?;
                    // remote_sock.bind((self.outgoing_local_addr.clone()+":0").parse().unwrap());
                    // return remote_sock.connect(host).await;
                }
            }
        }
        Err(std::io::Error::new(std::io::ErrorKind::Other, "No hosts returned!"))
    }
}