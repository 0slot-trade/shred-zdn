use std::{    
    sync::{Arc, atomic::AtomicUsize, RwLock},
    time::Duration,
    collections::HashMap,
};

use log::{info, warn, error};
use clap::{Parser};

use shred_zdn::common::{
    utils::init_env_logger,
    net_utils::tonic::generate_channel,
};
use shred_zdn::shred_zdn:: {
        zdn_ping::{
            sort_regions,
            resolve_nearest_n_region_addrs
        },
        args::{Opts, Protocol},
        stats::Stats,
        receiver::start_receivers,
        processor::start_processor,
        sniffer::start_sniffer,
        consts::{VERSION, HOST},
};
use zdn_proto::relay::{relay_client::RelayClient, RegisterRequest};

#[tokio::main]
async fn main() {
    init_env_logger();
    let opts = Opts::parse();
    let stats = Arc::new(Stats::new());
    let counter = Arc::new(AtomicUsize::new(0));
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    let (sender_sl, receiver_sl) = tokio::sync::mpsc::unbounded_channel();    

    // socket for forwarding shreds to validator    
    let forward_socket = tokio::net::UdpSocket::bind((std::net::Ipv4Addr::UNSPECIFIED, 0)).await.unwrap();    

    // network interface (lo0 for macOS's loopback, "lo" for Linux)
    let interface = opts.interface.as_str();
    let mut cap = pcap::Capture::from_device(interface)
        .expect("Failed to open device")
        .promisc(true)
        .immediate_mode(true)
        .snaplen(65535)
        .open()
        .expect("Capture init failed")
        .setnonblock()
        .expect("Failed to set non-blocking");
    
    // header length of different links：Ethernet=14，Loopback=NULL=4
    let header_len = match cap.get_datalink().0 {
        1 => 14,
        0 => 4,
        other => {
            warn!("Unknown link type {}, default to 4-byte", other);
            4
        }
    };
    let payload_offset = header_len + 20 + 8;
    let proto = match opts.protocol {
        Protocol::Udp => "udp",
        Protocol::Tcp => "tcp",
    };
    cap.filter(&format!("{} dst port {}", proto, opts.sniffer_port), true).unwrap();

    // region map return from server
    let region_map = Arc::new(RwLock::new(HashMap::<String, String>::new()));
    // set to request-region to request region map from server
    let region = Arc::new(RwLock::new("request-region".to_string()));

    // register runtime
    let runtime = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .thread_name("async")
            .enable_all()
            .build()
            .unwrap()
    );    
    let auth_key = opts.auth.clone();
    let port = opts.port;    
    let counter_clone = counter.clone();
    let region_clone = Arc::clone(&region);
    let region_map_clone = Arc::clone(&region_map);
    runtime.spawn(async move {
        let mut is_registered: bool = false;
        loop {
            let region_str = {
                        let guard = region_clone.read().unwrap();
                        guard.clone()
                    }; // release read lock
            // register to keep online.
            match generate_channel(HOST, None).await {
                Ok(channel) => {
                    let mut client = RelayClient::new(channel);
                    match client.register(RegisterRequest {
                        auth_header: auth_key.clone(),
                        version: VERSION.to_string(),
                        server_port: port as _,
                        region: region_str.clone(),
                    }).await {
                        Ok(response) => {
                            let inner = response.into_inner();
                            if region_str != "request-region" && !is_registered {
                                is_registered = true;
                                info!("Registered: {}, addr={}", inner.msg, inner.udp_address);
                            }
                            // get region map from server
                            if !inner.region_host_map.is_empty() {
                                let mut map = region_map_clone.write().unwrap();
                                *map = inner
                                    .region_host_map
                                    .iter()
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                    .collect();
                                if region_str == "request-region" {
                                    info!("get region map from server");
                                }                                
                            }                            
                        }
                        Err(err) => {                            
                            if err.message().contains("ipv6") {
                                error!("IPv6 not supported, please switch to IPv4");
                                std::process::exit(1);
                            }
                            if err.message().contains("auth_key") {
                                error!("Invalid auth key");
                                std::process::exit(2);
                            }
                            if err.message().contains("limit") {
                                error!("Exceeded IP registration limit");
                                std::process::exit(3);
                            }
                            warn!("Register failed: {}, retrying...", err);
                        }
                    }
                }
                Err(_) => warn!("Unable to connect to ZDN, retrying...")
            }
            
            let recent = counter_clone.load(std::sync::atomic::Ordering::Relaxed);
            // wait to see if any shreds received.
            tokio::time::sleep(Duration::from_secs(3)).await;
            let current = counter_clone.load(std::sync::atomic::Ordering::Relaxed);
            if current == recent {
                warn!("No recent shreds received...");
            }
        }
    });

    // wait until region_map is populated
    loop {
        {
            let map = region_map.read().unwrap();
            if !map.is_empty() {
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    let region_map_clone = {
        let map_guard = region_map.read().unwrap();        
        map_guard.clone() 
    }; // release read lock
    // find the nearest region
    let sorted_regions = sort_regions(&region_map_clone);
    let nearest_region = &sorted_regions[0];
    {
        let mut region_guard = region.write().unwrap();
        *region_guard = nearest_region.clone(); // write the nearest region back so that we can register using it.
    }
    info!("✅ Nearest region: {}", nearest_region);    

    //let region = find_nearest_region().expect("Unable to determine nearest region");
    let n = 3;
    let send_back_addrs = resolve_nearest_n_region_addrs(&region_map_clone, &sorted_regions, n);

    // sniff shreds of validator and send back to 0slot.trade to speed up.    
    info!("starting sniffer");
    let _ = start_sniffer(cap, payload_offset, Arc::new(forward_socket), Arc::new(send_back_addrs), receiver_sl).await;
    // receive shreds from 0slot.trade.    
    info!("starting receivers");
    let _ = start_receivers(&opts, &stats, &sender, &sender_sl).await;    
    // forward shreds to validator.
    info!("starting processor");    
    let _ = start_processor(&opts, &stats, receiver, &counter).await;

    // print stats
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        info!("stats: {}", stats.report());
    }
}
