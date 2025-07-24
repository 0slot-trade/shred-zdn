pub mod common {
    pub mod r#macro;
    pub mod utils;
    pub mod async_utils;
    pub mod net_utils {
        pub mod request;
        pub mod tonic;
    }
}

pub mod shred_zdn {           
    pub mod zdn_ping;
    pub mod args;
    pub mod stats;
    pub mod receiver;
    pub mod processor;
    pub mod sniffer;
    pub mod consts;
}
