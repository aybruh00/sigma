use std::env;
use listener::Manager;

mod listener;
mod proxy;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let mut ips = args.into_iter();
    ips.next();
    let ips: Vec<String> = ips.collect();

    let listener = Manager::new(ips, 9080 as u32);
    listener.listen().await;

    // println!("{:?}", ips);
}
