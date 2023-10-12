use tokio::net::{TcpStream, TcpSocket};
use std::cell::RefCell;

#[tokio::main]
async fn main(){
    // let s = returner().await;
    // println!("{}", s);
    let rc = RefCell::new(Some(5));
    println!("{:?} {:?}", rc, *rc.borrow_mut());
}