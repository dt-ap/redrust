use clap::Parser;

mod config;
mod core;
mod error;
mod server;

fn main() {
    let conf = config::Config::parse();
    println!("Starting the server!");
    server::async_tcp::run(conf).expect("Something's wrong!");
}
