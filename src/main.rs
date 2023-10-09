use std::env;
use listener::Manager;

mod listener;
mod proxy;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let mut ips = args.into_iter();
    ips.next();
    let incoming_port: u16 = ips.next().unwrap().parse().unwrap();
    let ips: Vec<String> = ips.collect();
    println!("{:?}", ips);

    let listener = Manager::new(ips, incoming_port);
    listener.listen().await;
}
