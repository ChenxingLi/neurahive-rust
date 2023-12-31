mod config_macro;

mod convert;
use config_macro::*;
use serde::Deserialize;
use std::ops::Deref;

build_config! {
    // network
    (network_dir, (String), "network".to_string())
    (network_listen_address, (String), "0.0.0.0".to_string())
    (network_libp2p_port, (u16), 1234)
    (network_target_peers, (usize), 3)
    (network_boot_nodes, (Vec<String>), vec![])
    (network_libp2p_nodes, (Vec<String>), vec![])
    (network_private, (bool), false)
    (network_disable_discovery, (bool), false)

    // log sync
    (blockchain_rpc_endpoint, (String), "http://127.0.0.1:8545".to_string())
    (log_contract_address, (String), "".to_string())
    (log_sync_start_block_number, (u64), 0)
    (confirmation_block_count, (u64), 12)
    (log_page_size, (u64), 1000)
    (max_cache_data_size, (usize), 100 * 1024 * 1024) // 100 MB
    (cache_tx_seq_ttl, (usize), 500)

    (rate_limit_retries, (u32), 100)
    (timeout_retries, (u32), 100)
    (initial_backoff, (u64), 500)

    // rpc
    (rpc_enabled, (bool), true)
    (rpc_listen_address, (String), "127.0.0.1:5678".to_string())
    (rpc_chunks_per_segment, (usize), 1024)
    (rpc_max_cache_file_size, (usize), 10*1024*1024) //10MB

    // chunk pool
    (chunk_pool_write_window_size, (usize), 4)
    (chunk_pool_max_cached_chunks_all, (usize), 4*1024*1024)    // 1G
    (chunk_pool_max_writings, (usize), 16)
    (chunk_pool_expiration_time_secs, (u64), 300)   // 5 minutes

    // db
    (db_dir, (String), "db".to_string())

    // misc
    (log_config_file, (String), "log_config".to_string())
    (log_directory, (String), "log".to_string())

    // mine
    (mine_contract_address, (String), "".to_string())
    (miner_id, (Option<String>), None)
    (miner_key, (Option<String>), None)

    // sync
    (find_peer_timeout_secs, (u64), 30)
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct NrhvConfig {
    pub raw_conf: RawConfiguration,

    pub sync: sync::Config,
}

impl Deref for NrhvConfig {
    type Target = RawConfiguration;

    fn deref(&self) -> &Self::Target {
        &self.raw_conf
    }
}

impl NrhvConfig {
    pub fn parse(matches: &clap::ArgMatches) -> Result<NrhvConfig, String> {
        let config_file = match matches.get_one::<String>("config") {
            Some(file) => file.as_str(),
            None => return Err("config file not specified".to_string()),
        };

        let mut config = config::Config::builder()
            .add_source(config::File::with_name(config_file))
            .build()
            .map_err(|e| format!("Failed to build config: {:?}", e))?
            .try_deserialize::<NrhvConfig>()
            .map_err(|e| format!("Failed to deserialize config: {:?}", e))?;

        config.raw_conf = RawConfiguration::parse(matches)?;

        Ok(config)
    }
}
