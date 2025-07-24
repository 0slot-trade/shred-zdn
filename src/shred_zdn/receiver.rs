use tokio::{
    net::UdpSocket, 
    select,
    sync::mpsc::UnboundedSender,
    task::JoinHandle,
};
use std::{
    io::ErrorKind,
    net::Ipv4Addr,
    sync::Arc,
};

use ahash::RandomState;
use solana_ledger::shred::ShredType;
use solana_sdk::signature::SIGNATURE_BYTES;
use crate::shred_zdn::args::{Opts, Source};
use crate::shred_zdn::stats::Stats;
use log::{error, info};

pub async fn start_receivers(
    opts: &Opts,
    stats: &Arc<Stats>,
    sender: &UnboundedSender<(Source, Vec<u8>, u64)>,
    sender_sl: &UnboundedSender<Vec<u8>>,
) -> Vec<JoinHandle<()>> {    
    let state = RandomState::new();
    let mut handles = Vec::new();

    let listeners = vec![
        Some((Source::Zdn, opts.port)),
        opts.reference.map(|p| (Source::Reference, p)),
    ];    
    for (source, port) in listeners.into_iter().flatten() {        
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port)).await.expect("bind failed");
        let state = state.clone();
        let stats = Arc::clone(&stats);
        let sender = sender.clone();
        let sender_sl = sender_sl.clone();
        info!("ready to receive shreds from {:?}:{}", source, port);
        handles.push(tokio::spawn(async move {
            let mut buf = vec![0u8; 2048];
            loop {
                select! {
                    // listen shreds from 0slot.trade.                    
                    recv = socket.recv_from(&mut buf) => {
                        let (len, _) = match recv {                            
                            Ok((len, _addr)) if len > 0 => (len, _addr),
                            Ok(_) => continue, // len is 0                           
                            Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
                            Err(e) => {
                                error!("Socket recv failed unrecoverably: {}", e);
                                std::process::exit(1);
                            }
                        };                        
                        let data_buf = &buf[..len];
                        stats.packets[source].fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                        let Some(sig) = data_buf.get(SIGNATURE_BYTES) else { continue };
                        let slice = match *sig {
                            b if b == u8::from(ShredType::Code) || b == u8::from(ShredType::Data) => {
                                stats.invalids[source].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                continue;
                            }
                            b => match b & 0xF0 {
                                0x40 | 0x60 => data_buf.get(..1228),
                                0x70 => data_buf.get(..1228-64),
                                0x80|0x90 => data_buf.get(..1203),
                                0xB0 => data_buf.get(..1203-64),
                                _ => {
                                    stats.invalids[source].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    continue;
                                }
                            },
                        };
                        let Some(slice) = slice else {
                            stats.invalids[source].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            continue;
                        };

                        let hash = state.hash_one(slice);                        
                        sender.send((source, data_buf.to_vec(), hash)).unwrap();
                        sender_sl.send(data_buf.to_vec()).unwrap();                        
                    }                    
                }
            }
        }));
    }    
    handles
}
