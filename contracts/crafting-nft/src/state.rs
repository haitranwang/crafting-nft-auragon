use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use cw_storage_plus::{Item, Map, Deque};

#[cw_serde]
pub struct Config {
    pub nois_proxy: Addr,
    pub auragon_collection: Addr,
    pub shield_collection: Addr,
}

#[cw_serde]
pub struct GemInfo {
    pub nft_id: String,
    pub nft_contract: Addr,
}

#[cw_serde]
pub struct UsersInQueue {
    pub user_addr: Addr,
    pub gem_base: GemInfo,
    pub gem_materials: Vec<GemInfo>,
    pub shield_id: Option<String>,
    pub timestamp: Timestamp,
}

#[cw_serde]
#[derive(Default)]
pub struct GemMetadata {
    pub color: String,
    pub level: u8,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const RANDOM_SEED: Item<[u8; 32]> = Item::new("random seed");

pub const LATEST_TOKEN_ID: Item<u64> = Item::new("current token id");

// DeQueue to store the gem forging requests from users
pub const USERS_IN_QUEUE: Deque<UsersInQueue> = Deque::new("users_in_queue");
