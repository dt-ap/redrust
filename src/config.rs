use clap::Parser;

/// Program to simulate Redis functionalities
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Host for the server
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    /// Port number for the server
    #[arg(long, default_value_t = 7379)]
    pub port: u16,

    #[arg(long, default_value_t = 5)]
    pub keys_limit: i32,

    #[arg(long, default_value = "simple-first")]
    pub eviction_strategy: String,

    #[arg(long, default_value = "./redrust-master.aof")]
    pub aof_file: String,
}
