use std::env;
use listener::Manager;
use tokio::io;

mod listener;
mod proxy;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut ips = args.into_iter();
    ips.next();
    let incoming_port: u16 = ips.next().unwrap().parse().unwrap();
    let ips: Vec<String> = ips.collect();
    println!("{:?}", ips);

    // console_subscriber::init();
    let listener = Manager::new(ips, incoming_port);
    tokio::spawn(async move{
        listener.listen().await;
    }).await?;
    Ok(())
}
