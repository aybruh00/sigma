use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use crate::proxy::HttpProxyTunnel;

pub struct Manager {
    ips: Vec<String>,
    listener_port: u32
}

impl Manager{
    pub fn new(ips: Vec<String>, port: u32) -> Manager{
        Manager{ips: ips, listener_port: port}
    }

    pub async fn listen(&self) {

        let listener = TcpListener::bind(format!("{}:{}", "0.0.0.0", self.listener_port)).await.unwrap();

        loop{
            let (conn_sock, _conn_addr) = listener.accept().await.unwrap();
            println!("Connected from {:?}", _conn_addr);
            Self::process(conn_sock, _conn_addr);
        }
    }

    fn process(_socket: TcpStream, _addr: SocketAddr){
        // let mut buf = [0u8; 2048];
        // _socket.peek(&mut buf).await;
    }
}
