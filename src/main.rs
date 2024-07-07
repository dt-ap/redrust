use clap::Parser;

mod common;
mod config;
mod core;
mod data;
mod error;
mod server;

fn main() {
    let conf = config::Config::parse();

    println!("Starting the server!");

    server::async_tcp::run(conf).expect("Something's wrong!");
}
