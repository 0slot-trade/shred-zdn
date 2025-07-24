use enum_map::{enum_map, EnumMap};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use crate::shred_zdn::args::Source;

pub struct Stats {
    pub packets: EnumMap<Source, AtomicUsize>,
    pub invalids: EnumMap<Source, AtomicUsize>,
    pub firsts: EnumMap<Source, AtomicUsize>,
    pub forwarded: AtomicUsize,
    pub nanos: AtomicU64,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            packets: enum_map! { _ => AtomicUsize::new(0) },
            invalids: enum_map! { _ => AtomicUsize::new(0) },
            firsts: enum_map! { _ => AtomicUsize::new(0) },
            forwarded: AtomicUsize::new(0),
            nanos: AtomicU64::new(0),
        }
    }

    pub fn report(&self) -> String {
        let zdn_packet = self.packets[Source::Zdn].swap(0, Ordering::Relaxed);
        let reference_packet = self.packets[Source::Reference].swap(0, Ordering::Relaxed);
        let zdn_invalid = self.invalids[Source::Zdn].swap(0, Ordering::Relaxed);
        let reference_invalid = self.invalids[Source::Reference].swap(0, Ordering::Relaxed);
        let zdn_first = self.firsts[Source::Zdn].swap(0, Ordering::Relaxed);
        let reference_first = self.firsts[Source::Reference].swap(0, Ordering::Relaxed);
        let forwarded = self.forwarded.swap(0, Ordering::Relaxed);
        let nanos = self.nanos.swap(0, Ordering::Relaxed);

        format!(
            "zdn-packet {zdn_packet}, reference-packet {reference_packet}, \
            zdn-invalid {zdn_invalid}, reference-invalid {reference_invalid}, \
            zdn-first {zdn_first}, reference-first {reference_first}, \
            forwarded {forwarded}, \
            ms {:.2}",
            nanos as f64 / 1e6
        )
    }
}
