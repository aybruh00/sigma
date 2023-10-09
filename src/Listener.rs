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

        loop{
            let (conn_sock, _conn_addr) = listener.accept().await.unwrap();
            println!("Connected from {:?}", _conn_addr);
            let ipaddr = self.ips[0].clone();
            tokio::spawn( async move {
                process(conn_sock, _conn_addr, ipaddr).await;
            });
            // Self::process(conn_sock, _conn_addr);
        }
    }
}

async fn process(_socket: TcpStream, _addr: SocketAddr, local_addr: String){
    let mut proxy = HttpProxyTunnel{
        incoming_sock: _socket,
        incoming_remote_addr: _addr.to_string(),
        outgoing_local_addr: local_addr,
        buf: Box::new([0u8; 4096])
    };
    proxy.start().await;
}
