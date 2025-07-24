use std::{time::Duration, net::{IpAddr, Ipv4Addr}};

pub fn generate_client(ip: Option<Ipv4Addr>) -> reqwest::Client {
    reqwest::ClientBuilder::new()
        .local_address(ip.map(IpAddr::V4)) 
        .tcp_nodelay(true) 
        .tcp_keepalive(Duration::from_secs(1))
        .http2_keep_alive_interval(Duration::from_secs(1)) 
        .http2_keep_alive_timeout(Duration::from_secs(5))
        .http2_keep_alive_while_idle(true)
        .pool_idle_timeout(None) 
        .use_rustls_tls()
        .build().unwrap()
}