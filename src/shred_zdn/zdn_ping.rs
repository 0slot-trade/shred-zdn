use std::{
    process::Command,
    sync::Mutex,
    collections::HashMap,
    net::{SocketAddr, ToSocketAddrs},
};
use rayon::prelude::*;

/// return the region with the lowest average ping latency.
pub fn find_nearest_region(host_map: &HashMap<String, String>) -> Option<String> {
    //let host_map = get_region_host_map();
    let results = Mutex::new(Vec::new());

    host_map.par_iter().for_each(|(region, host)| {
        let avg = ping_avg_latency(host, 4);
        log::info!("Region: {}, Host: {}, Avg Latency: {:?}", region, host, avg);
        if let Some(ms) = avg {
            results.lock().unwrap().push((region.to_string(), ms));
        }
    });

    results
        .lock()
        .unwrap()
        .iter()
        .min_by_key(|(_, ms)| *ms)
        .map(|(region, _)| region.clone())
}

/// regions order by average ping latency asc.
pub fn sort_regions(host_map: &HashMap<String, String>) -> Vec<String> {    
    let results = Mutex::new(Vec::new());

    // ping hosts
    host_map.par_iter().for_each(|(region, host)| {
        let avg = ping_avg_latency(host, 4);
        // log::info! aple_log::info!("Region: {}, Host: {}, Avg Latency: {:?}", region, host, avg);
        if let Some(ms) = avg {
            results.lock().unwrap().push((region.to_string(), ms));
        }
    });

    let mut collected_results = results.into_inner().unwrap();

    // sort by ping latency asc
    collected_results.sort_by_key(|(_, ms)| *ms);
    collected_results
        .into_iter()
        .map(|(region, _)| region)
        .collect()
}

pub fn resolve_nearest_n_region_addrs(
    host_map: &HashMap<String, String>,
    sorted_regions: &[String],
    n: usize,
) -> Vec<SocketAddr> {
    let port = 8002;
    let mut result = Vec::new();

    for region_name in sorted_regions.iter().take(n) {
        // get host domain name by region name
        if let Some(host) = host_map.get(region_name) {
            log::info!(
                "Resolving domain for top region '{}': {}:{}",
                region_name,
                host,
                port
            );
            // Resolve domain + port into a SocketAddr.
            match (host.as_str(), port).to_socket_addrs() {
                Ok(addrs_iter) => {
                    // A domain name may resolve to multiple IP addresses (e.g., both IPv4 and IPv6).
                    result.extend(addrs_iter);
                }
                Err(e) => {
                    log::warn!(
                        "Failed to resolve domain name '{}:{}': {}",
                        host,
                        port,
                        e
                    );
                }
            }
        } else {
            log::warn!(
                "Region '{}' from sorted list not found in host_map.",
                region_name
            );
        }
    }

    result
}


/// exec ping -c <count> <host>, and then return the average latency.
fn ping_avg_latency(host: &str, count: usize) -> Option<u128> {
    let output = Command::new("ping")
        .args(["-c", &count.to_string(), host])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("rtt") || line.contains("round-trip") {
            // e.g：rtt min/avg/max/mdev = 23.456/56.789/90.123/12.345 ms
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() >= 2 {
                let stats = parts[1].trim().split('/').collect::<Vec<&str>>();
                if stats.len() >= 2 {
                    return stats[1].parse::<f64>().ok().map(|v| v.round() as u128);
                }
            }
        }
    }
    None
}
