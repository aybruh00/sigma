use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use crate::proxy::HttpProxyTunnel;

pub struct Manager {
    ips: Vec<String>,
    listener_port: u16
}

impl Manager{
    pub fn new(ips: Vec<String>, port: u16) -> Manager{
        Manager{ips: ips, listener_port: port}
    }

    pub async fn listen(&self) {

        let listener = TcpListener::bind(format!("{}:{}", "0.0.0.0", self.listener_port)).await.unwrap();
        let mut idx = 0;

        loop{
            idx = (idx+1)%self.ips.len();
            let (conn_sock, _conn_addr) = listener.accept().await.unwrap();
            println!("Connected from {:?}", _conn_addr);

            let ipaddr = self.ips[idx].clone();
            tokio::spawn( async move {
                process(conn_sock, _conn_addr, ipaddr).await;
            });
        }
    }
}

async fn process(_socket: TcpStream, _addr: SocketAddr, local_addr: String){
    let (r, w) = _socket.into_split();
    let mut proxy = HttpProxyTunnel{
        // local_r: r,
        // local_w: w,
        // outgoing_sock: None,
        // incoming_remote_addr: _addr.to_string(),
        outgoing_local_addr: local_addr,
        buf: vec![0u8; 4096]
    };
    proxy.start(r, w).await;
}
