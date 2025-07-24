use std::{
    collections::HashMap,
    net::Ipv4Addr,
    sync::{atomic::{AtomicUsize, Ordering}, Arc},
    time::{Duration, Instant},
};

use tokio::{net::UdpSocket, select, sync::mpsc::UnboundedReceiver, time};
use log::{error};
use nohash_hasher::BuildNoHashHasher;

use crate::shred_zdn::args::{Opts, Source};
use crate::shred_zdn::stats::Stats;

pub async fn start_processor(
    opts: &Opts,
    stats: &Arc<Stats>,
    mut receiver: UnboundedReceiver<(Source, Vec<u8>, u64)>,
    counter: &Arc<AtomicUsize>,
) {
    let mut current = HashMap::<u64, Source, BuildNoHashHasher<u64>>::default();
    let mut preparing = HashMap::<u64, Source, BuildNoHashHasher<u64>>::default();
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await.expect("bind failed");
    let addrs = opts.forwards.clone();
    let mut rotate_interval = time::interval(Duration::from_secs(15));
    let stats = Arc::clone(&stats);
    let counter = Arc::clone(&counter);

    tokio::spawn(async move {
        loop {
            select! {
                // process shreds from 0slot.trade
                Some((source, buf, hash)) = receiver.recv() => {
                    let now = Instant::now();
                    scopeguard::defer! {
                        stats.nanos.fetch_add(now.elapsed().as_nanos() as u64, Ordering::Relaxed);
                    }

                    if current.contains_key(&hash) {
                        continue;
                    }
                    current.insert(hash, source);
                    preparing.insert(hash, source);

                    stats.firsts[source].fetch_add(1, Ordering::Relaxed);
                    if matches!(source, Source::Zdn) {
                        // forward shreds from 0slot.trade to validators
                        for addr in &addrs {
                            match socket.send_to(&buf, addr).await {
                                Ok(_) => (),
                                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                    error!("sys buff is full.");
                                }
                                Err(e) => error!("Send failed: {}", e),
                            }
                        }
                        stats.forwarded.fetch_add(1, Ordering::Relaxed);
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                },

                _ = rotate_interval.tick() => {
                    std::mem::swap(&mut current, &mut preparing);
                    preparing.clear();
                    // info!("swap current and preparing");
                }
            }
        }
    });
}
