use std::net::SocketAddr;
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum Protocol {
    Udp,
    Tcp,
}

#[derive(Parser, Debug, Clone)]
#[command(about = "Receive Shreds from 0slot.trade")]
pub struct Opts {
    /// zdn auth key
    #[clap(long)]
    pub auth: String,

    /// port to receive shreds from 0slot.trade
    #[clap(long)]
    pub port: u16,    

    /// The network interface to sniff for the local validator's traffic (e.g., en0). If the validator is on the same host, use a loopback interface, such as `lo`
    #[clap(long)]
    pub interface: String,

    /// The local validator's shred port to sniff.
    #[clap(long)]
    pub sniffer_port: u16,

    /// Protocol (udp or tcp), case-insensitive
    #[clap(long, default_value = "udp")]
    pub protocol: Protocol,

    /// forward addresses, comma-separated, at lease one
    #[clap(long, value_delimiter = ',', required = true, num_args = 1..)]
    pub forwards: Vec<SocketAddr>,

    /// reference shred-stream port
    #[clap(long)]
    pub reference: Option<u16>,
}

#[derive(enum_map::Enum, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Source {
    Zdn,
    Reference,
}


