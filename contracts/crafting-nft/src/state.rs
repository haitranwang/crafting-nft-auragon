use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub is_advanced_randomness: bool,
    pub nois_proxy: Addr,
}

#[cw_serde]
pub struct GemInfo {
    pub nft_id: u64,
    pub nft_contract: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const RANDOM_SEED: Item<[u8; 32]> = Item::new("random seed");
