use std::{net::{IpAddr, Ipv4Addr}, time::Duration};
use tonic::transport;

pub async fn generate_channel(
    entry: impl Into<String>, 
    ip: Option<Ipv4Addr>, 
) -> Result<transport::Channel, transport::Error> {
    let entry = entry.into();
    if entry.starts_with("http://") || entry.starts_with("https://") { 
        let mut endpoint = transport::Endpoint::try_from(entry.clone())? 
            .tcp_nodelay(true) 
            .tcp_keepalive(Some(Duration::from_secs(1))) 
            .http2_keep_alive_interval(Duration::from_secs(1)) 
            .keep_alive_timeout(Duration::from_secs(10)) 
            .keep_alive_while_idle(true) 
            .initial_connection_window_size(Some(8 * 1024 * 1024)) 
            .initial_stream_window_size(Some(4 * 1024 * 1024)) 
            .buffer_size(64 * 1024); 
        if entry.starts_with("https://") {
            endpoint = endpoint 
                .tls_config(transport::ClientTlsConfig::new())?; 
        }
        endpoint.connect_with_connector({            
            let mut connector = hyper::client::HttpConnector::new();
            connector.enforce_http(false);
            connector.set_nodelay(true);
            connector.set_keepalive(Some(Duration::from_secs(1)));        
            connector.set_local_address(ip.map(|ip| IpAddr::V4(ip)));            
            connector
        }).await        
    } else { 
        transport::Endpoint::try_from("http://[::]:0") 
            .unwrap() 
            .buffer_size(64 * 1024) 
            .connect_with_connector(tower::service_fn(                
                move |_| tokio::net::UnixStream::connect(entry.clone())
            )).await
    }
}