use cosmwasm_schema::{cw_serde, QueryResponses};
use nois::NoisCallback;

use crate::state::{GemInfo, Config};


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
    ForgeGem { is_success: bool },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
}