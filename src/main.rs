use clap::Parser;

mod config;
mod error;
mod server;

fn main() {
    let conf = config::Config::parse();
    println!("Starting the server!");
    server::sync_tcp::run(conf).expect("Something's wrong!");
}
