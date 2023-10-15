#![feature(cell_leak)]
use std::env;
use listener::Manager;

mod listener;
mod proxy;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    // let subscriber = tracing_subscriber::FmtSubscriber::new();
    // use that subscriber to process traces emitted after this point
    // tracing::subscriber::set_global_default(subscriber);

    let args: Vec<String> = env::args().collect();
    let mut ips = args.into_iter();
    ips.next();
    let incoming_port: u16 = ips.next().unwrap().parse().unwrap();
    let ips: Vec<String> = ips.collect();
    println!("{:?}", ips);

    console_subscriber::init();
    let listener = Manager::new(ips, incoming_port);
    tokio::spawn(async move{
        listener.listen().await;
    }).await;

}
