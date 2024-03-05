use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp, Uint128, Decimal};
use cw_storage_plus::{Item, Map, Deque};

#[cw_serde]
pub struct Config {
    pub nois_proxy: Addr,
    pub dragon_collection: Addr,
    pub auragon_collection: Addr,
    pub shield_collection: Addr,
}

#[cw_serde]
pub struct GemInfo {
    pub nft_id: String,
    pub nft_contract: Addr,
}

#[cw_serde]
pub struct UserInfo {
    pub user_addr: Addr,
    pub gem_base: GemInfo,
    pub gem_materials: Vec<GemInfo>,
    pub shield_id: Option<String>,
    pub timestamp: Timestamp,
}

#[cw_serde]
pub struct RandomJob {
    pub player: Addr,
    pub timestamp: Timestamp,
}

#[cw_serde]
#[derive(Default)]
pub struct GemMetadata {
    pub color: String,
    pub level: u8,
    pub work_power: Decimal,
}

pub const WHITE_GEM_WORK_POWER: Item<[Decimal; 7]> = Item::new("white gem work power");
// [Decimal; 7] =[
//     Decimal::from_str("2").unwrap(),
//     Decimal::from_str("3").unwrap(),
//     Decimal::from_str("5").unwrap(),
//     Decimal::from_str("8").unwrap(),
//     Decimal::from_str("13").unwrap(),
//     Decimal::from_str("21").unwrap(),
//     Decimal::from_str("34").unwrap(),
// ];

pub const BLUE_GEM_WORK_POWER: Item<[Decimal; 7]> = Item::new("blue gem work power");
// [Decimal; 7] =[
//     Decimal::from_str("22.5").unwrap(),
//     Decimal::from_str("33.75").unwrap(),
//     Decimal::from_str("56.25").unwrap(),
//     Decimal::from_str("90").unwrap(),
//     Decimal::from_str("146.25").unwrap(),
//     Decimal::from_str("236.25").unwrap(),
//     Decimal::from_str("382.5").unwrap(),
// ];

pub const GOLD_GEM_WORK_POWER: Item<[Decimal; 7]> = Item::new("gold gem work power");
// [Decimal; 7] =[
//     Decimal::from_str("120").unwrap(),
//     Decimal::from_str("180").unwrap(),
//     Decimal::from_str("300").unwrap(),
//     Decimal::from_str("480").unwrap(),
//     Decimal::from_str("780").unwrap(),
//     Decimal::from_str("1260").unwrap(),
//     Decimal::from_str("2040").unwrap(),
// ];

pub const RED_GEM_WORK_POWER: Item<[Decimal; 7]> = Item::new("red gem work power");
// [Decimal; 7] =[
//     Decimal::from_str("360").unwrap(),
//     Decimal::from_str("540").unwrap(),
//     Decimal::from_str("900").unwrap(),
//     Decimal::from_str("1440").unwrap(),
//     Decimal::from_str("2340").unwrap(),
//     Decimal::from_str("3780").unwrap(),
//     Decimal::from_str("6120").unwrap(),
// ];

pub const GEM_RATIO: Item<[Decimal; 4]> = Item::new("gem ratio");
// [Decimal; 4] = [
//     Decimal::from_str("0.9").unwrap(),
//     Decimal::from_str("0.08").unwrap(),
//     Decimal::from_str("0.015").unwrap(),
//     Decimal::from_str("0.005").unwrap(),
// ];

pub const GEM_WORK_LOAD: Item<[Decimal; 6]> = Item::new("gem work load");
// [Decimal; 6] = [
//     Decimal::from_str("3").unwrap(),
//     Decimal::from_str("5").unwrap(),
//     Decimal::from_str("8").unwrap(),
//     Decimal::from_str("13").unwrap(),
//     Decimal::from_str("21").unwrap(),
//     Decimal::from_str("34").unwrap(),
// ];

pub const CONFIG: Item<Config> = Item::new("config");

pub const RANDOM_SEED: Item<[u8; 32]> = Item::new("random seed");

pub const RANDOM_JOBS: Map<String, RandomJob> = Map::new("random jobs");

pub const AURAGON_LATEST_TOKEN_ID: Item<u64> = Item::new("auragon latest token id");

pub const SHIELD_LATEST_TOKEN_ID: Item<u64> = Item::new("shield latest token id");

// DeQueue to store the gem forging requests from users
pub const USERS_IN_QUEUE: Deque<UserInfo> = Deque::new("users_in_queue");

// Current Queue ID
pub const CURRENT_QUEUE_ID: Item<u64> = Item::new("current queue id");
