use std::time::{Duration, SystemTime};

// init env-logger
pub fn init_env_logger() {
    let mut builder = env_logger::Builder::new();

    // set filter level
    builder // set level to info
        .filter_level(log::LevelFilter::Info)
        .filter(Some("ethers_providers"), log::LevelFilter::Warn)
        .filter(Some("solana_pubsub_client"), log::LevelFilter::Warn)
        .filter(Some("solana_program_test"), log::LevelFilter::Warn)
        .filter(Some("solana_accounts_db"), log::LevelFilter::Warn)
        .filter(Some("solana_runtime"), log::LevelFilter::Warn)
        .filter(Some("solana_metrics"), log::LevelFilter::Warn)        
        .filter(Some("tarpc"), log::LevelFilter::Warn);
    
    if let Ok(rust_log) = std::env::var(env_logger::DEFAULT_FILTER_ENV) {
        builder.parse_filters(rust_log.as_str());
    }

    // log output to Stdout
    builder.target(env_logger::Target::Stdout);

    // set log format
    builder.format(|buf, record| {
        use std::io::Write;
        let style = buf.default_level_style(record.level());
        writeln!(buf, "[{} {}{}{} {}:{}] {}",            
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.6f"),            
            style.render(), record.level(), style.render_reset(),            
            record.file().unwrap_or("unknown"), record.line().unwrap_or(0),            
            record.args(),
        )
    });
        
    builder.init();
}

#[inline]
pub fn last_n_chars(s: &str, n: usize) -> &str {
    &s[s.len().saturating_sub(n)..]
}

#[inline]
pub fn current_ns() -> usize {
    let ns = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap() 
        .as_nanos() % 1_000_000_000;
    ns as usize
}

#[inline]
pub fn diff_time(start: SystemTime, stop: SystemTime) -> i128 {
    if start < stop {
        stop.duration_since(start)
            .unwrap().as_micros() as i128
    } else {
        start.duration_since(stop)
            .unwrap().as_micros() as i128 * -1
    }
}

pub fn lerp_duration(start: Duration, end: Duration, t: f32) -> Duration {    
    let start_secs = start.as_secs_f32();
    let end_secs = end.as_secs_f32();
    
    let lerp_secs = (1.0 - t) * start_secs + t * end_secs;

    Duration::from_secs_f32(lerp_secs)
}

#[inline(always)]
pub fn time_to_string(time: impl Into<chrono::DateTime<chrono::Utc>>) -> String {
    time.into().to_rfc3339()
}
