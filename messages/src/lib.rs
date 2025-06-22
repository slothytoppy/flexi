pub mod client;
pub mod server;

pub use client::*;
pub use server::*;

pub const CONNECTION_PATH: &str = "/tmp/flexi.sock";
