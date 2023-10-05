use tokio::net::{TcpListener, TcpStream};

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
            println!("Connected {:?}", _conn_addr);
            tokio::spawn(async move {
                Self::process(conn_sock);
            });
        }
    }

    fn process(_socket: TcpStream){

    }
}
