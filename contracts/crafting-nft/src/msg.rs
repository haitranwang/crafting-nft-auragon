use cosmwasm_schema::{cw_serde, QueryResponses};
use nois::NoisCallback;

use crate::state::{GemInfo, Config};


/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    // must be hex string and has length 64
    pub random_seed: String,
    pub is_advanced_randomness: bool,
    // bench32 string address
    pub nois_proxy: String,
    // Auragon Ball NFT Collection address
    pub auragon_collection: String,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    // A map maps nft id with NFT contract address
    ForgingGem {
        gem_base: GemInfo,
        gem_materials: Vec<GemInfo>,
        shield: Option<GemInfo>,
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
}