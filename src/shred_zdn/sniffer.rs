use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};

use ahash::RandomState;
use log::{error, info, warn};
use nohash_hasher::BuildNoHashHasher;
use pcap::Capture;
use solana_ledger::shred::Shred;
use tokio::{
    net::UdpSocket,
    sync::mpsc::{self, UnboundedReceiver},
    time::{interval, MissedTickBehavior},
};

use crate::shred_zdn::{args::Source};

pub async fn start_sniffer(
    cap: Capture<pcap::Active>,
    payload_offset: usize,
    socket: Arc<UdpSocket>,
    addrs: Arc<Vec<SocketAddr>>,
    mut zdn_receiver: UnboundedReceiver<Vec<u8>>,
) {
    let mut current = HashMap::<u64, Source, BuildNoHashHasher<u64>>::default();
    let mut preparing = HashMap::<u64, Source, BuildNoHashHasher<u64>>::default();
    let mut total_send_back_count = 0;
    let state = RandomState::new();

    info!("✅ Packet listener started.");
    info!(
        "Send back to: {}",
        addrs.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ")
    );

    // channel for pcap
    let (pcap_tx, mut pcap_rx) = mpsc::unbounded_channel::<Vec<u8>>();

    // pcap thread
    std::thread::spawn(move || {
        let mut cap = cap;        
        info!("sniffer thread started");
        
        loop {
            match cap.next_packet() {
                Ok(packet) => {
                    // send to forwarder
                    if pcap_tx.send(packet.data.to_vec()).is_err() {
                        info!("Pcap channel closed, stopping pcap thread");
                        break;
                    }
                }
                Err(pcap::Error::NoMorePackets) | Err(pcap::Error::TimeoutExpired) => {
                    continue;
                }
                Err(e) => {
                    error!("❌ Packet listener error: {:?}", e);
                    break;
                }
            }
        }
        info!("🛑 Pcap thread terminated");
    });
    
    let mut interval_timer = interval(Duration::from_secs(15));
    interval_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut stats_timer = interval(Duration::from_secs(60));
    stats_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    
    tokio::spawn(async move {
        loop {
            tokio::select! {
                // 0slot.trade shreds for checking duplication
                zdn_data = zdn_receiver.recv() => {
                    match zdn_data {
                        Some(shred_data) if !shred_data.is_empty() => {
                            let hash = state.hash_one(&shred_data);
                            if !current.contains_key(&hash) {
                                current.insert(hash, Source::Zdn);
                                preparing.insert(hash, Source::Zdn);
                            }
                        }
                        Some(_) => {}, // no data
                        None => {
                            info!("ZDN receiver channel closed");
                            break;
                        }
                    }
                }

                // process pcap data
                packet_data = pcap_rx.recv() => {
                    match packet_data {
                        Some(data) => {
                            // invalid packet
                            if data.len() <= payload_offset {
                                continue;
                            }
                            
                            let udp_payload = &data[payload_offset..];
                            
                            // check duplication
                            match Shred::new_from_serialized_shred(udp_payload.to_vec()) {
                                Ok(shred) => {
                                    let hash = state.hash_one(shred.payload());
                                    if current.contains_key(&hash) {
                                        // warn!("duplicated");
                                        continue;
                                    }
                                    
                                    total_send_back_count += 1;
                                    
                                    // forward to validators
                                    let send_futures: Vec<_> = addrs.iter().map(|&addr| {
                                        let socket = Arc::clone(&socket);
                                        let payload = udp_payload.to_vec();
                                        async move {
                                            match socket.send_to(&payload, addr).await {
                                                Ok(_) => {},
                                                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                                    error!("System buffer full for {}", addr);
                                                }
                                                Err(e) => {
                                                    error!("Send to {} failed: {}", addr, e);
                                                }
                                            }
                                        }
                                    }).collect();
                                    
                                    // forwarding in threads
                                    tokio::spawn(async move {
                                        futures::future::join_all(send_futures).await;
                                    });
                                }
                                Err(_) => {
                                    warn!("⚠️ Invalid Shred data, length = {} bytes", udp_payload.len());
                                }
                            }
                        }
                        None => {
                            info!("Pcap receiver channel closed");
                            break;
                        }
                    }
                }

                // swap every 15secs
                _ = interval_timer.tick() => {
                    std::mem::swap(&mut current, &mut preparing);
                    preparing.clear();
                }

                // stats
                _ = stats_timer.tick() => {
                    info!(
                        "📊 Total send back count last one minute = {} to addrs: {}",
                        total_send_back_count,
                        addrs.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ")
                    );
                    total_send_back_count = 0;
                }
            }
        }
    });    
}