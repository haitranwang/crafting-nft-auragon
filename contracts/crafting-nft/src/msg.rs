use cosmwasm_schema::{cw_serde, QueryResponses};
use nois::NoisCallback;

use crate::state::{Config, GemInfo, GemMetadata, UserInfo};


/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    // must be hex string and has length 64
    pub random_seed: String,
    // bench32 string address
    pub nois_proxy: String,
    // Dragon Gem NFT Collection address
    pub dragon_collection: String,
    // Auragon Ball NFT Collection address
    pub auragon_collection: String,
    // Shield NFT Collection address
    pub shield_collection: String,
    // White Gem Work Power
    pub white_gem_work_power: [String; 7],
    // White Gem uri
    pub white_gem_uri: [String; 7],
    // Blue Gem Work Power
    pub blue_gem_work_power: [String; 7],
    // Blue Gem uri
    pub blue_gem_uri: [String; 7],
    // Gold Gem Work Power
    pub gold_gem_work_power: [String; 7],
    // Gold Gem uri
    pub gold_gem_uri: [String; 7],
    // Red Gem Work Power
    pub red_gem_work_power: [String; 7],
    // Red Gem uri
    pub red_gem_uri: [String; 7],
    // Gem Ratio
    pub gem_ratio: [String; 7],
    // Gem work load
    pub gem_work_load: [String; 7],
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    // All users join the queue to forge gem
    JoinQueue {
        gem_base: GemInfo,
        gem_materials: Vec<GemInfo>,
        shield_id: Option<String>,
    },
    // Forging gem
    ForgeGem {
        user_list: Vec<UserInfo>,
    },
    // Nois callback
    NoisReceive {
        callback: NoisCallback,
    },
    // Update collection
    UpdateCollection {
        dragon_collection: Option<String>,
        auragon_collection: Option<String>,
        shield_collection: Option<String>,
    },
    MintAuragonGem {
        owner: String,
        token_uri: String,
        extension: GemMetadata,
    },
    MintShieldGem {
        owner: String,
        token_uri: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    // Random seed
    #[returns(String)]
    RandomSeed {},
}