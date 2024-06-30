use clap::Parser;

/// Program to simulate Redis functionalities
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Host for the server
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    /// Port number for the server
    #[arg(long, default_value_t = 7379)]
    pub port: u16,
}
