#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, has_coins, to_json_binary, wasm_execute, Addr, Api, BalanceResponse, BankMsg, BankQuery, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, QueryRequest, Response, StdResult, Storage, Timestamp, Uint128, WasmMsg
};
use cw2::set_contract_version;

use cw721::Cw721ExecuteMsg;
use cw721_base::ExecuteMsg as Cw721BaseExecuteMsg;

use cw20::Cw20ExecuteMsg;
use nois::randomness_from_str;

use crate::{error::ContractError, msg::{ExecuteMsg, InstantiateMsg}, state::{Config, GemInfo, CONFIG, RANDOM_SEED}};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:wheel-of-fortune";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_TEXT_LENGTH: usize = 253;
const MAX_VEC_ITEM: usize = 65536;
const MAX_SPINS_PER_TURN: u32 = 10;
const DEFAULT_ACTIVATE: bool = false;

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let nois_proxy = addr_validate(deps.api, &msg.nois_proxy)?;

    let config = Config {
        is_advanced_randomness: msg.is_advanced_randomness,
        nois_proxy,
    };
    CONFIG.save(deps.storage, &config)?;

    // save the init RANDOM_SEED to the storage
    let randomness = randomness_from_str(msg.random_seed).unwrap();
    RANDOM_SEED.save(deps.storage, &randomness)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ForgingGem {
            gem_base,
            gem_materials,
            shield,
        } => execute_forging_gem(deps, env, info, gem_base, gem_materials, shield),
    }
}

pub fn execute_forging_gem(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    gem_base: GemInfo,
    gem_materials: Vec<GemInfo>,
    shield: Option<GemInfo>,
) -> Result<Response, ContractError> {
    let mut res = Response::new();
    // Approve the gem_base, gem_materials and shield NFT contracts to transfer the NFT to this contract
    let approve_gem_base = wasm_execute(
        gem_base.nft_contract.clone(),
        &Cw721ExecuteMsg::Approve {
            spender: env.contract.address.to_string(),
            token_id: gem_base.nft_id.to_string(),
            expires: None,
        },
        vec![],
    )?;

    res = res.add_message(approve_gem_base);

    for gem_material in gem_materials {
        let approve_gem_material = wasm_execute(
            gem_material.nft_contract.clone(),
            &Cw721ExecuteMsg::Approve {
                spender: env.contract.address.to_string(),
                token_id: gem_material.nft_id.to_string(),
                expires: None,
            },
            vec![],
        )?;
        res = res.add_message(approve_gem_material);
    }

    if let Some(shield) = shield {
        let approve_shield = wasm_execute(
            shield.nft_contract.clone(),
            &Cw721ExecuteMsg::Approve {
                spender: env.contract.address.to_string(),
                token_id: shield.nft_id.to_string(),
                expires: None,
            },
            vec![],
        )?;
        res = res.add_message(approve_shield);
    }

    // Transfer the gem_base, gem_materials and shield NFTs to this contract
    let transfer_gem_base = wasm_execute(
        gem_base.nft_contract.clone(),
        &Cw721ExecuteMsg::TransferNft {
            recipient: env.contract.address.to_string(),
            token_id: gem_base.nft_id.to_string(),
        },
        vec![],
    )?;

    res = res.add_message(transfer_gem_base);

    for gem_material in gem_materials {
        let transfer_gem_material = wasm_execute(
            gem_material.nft_contract.clone(),
            &Cw721ExecuteMsg::TransferNft {
                recipient: env.contract.address.to_string(),
                token_id: gem_material.nft_id.to_string(),
            },
            vec![],
        )?;
        res = res.add_message(transfer_gem_material);
    }

    if let Some(shield) = shield {
        let transfer_shield = wasm_execute(
            shield.nft_contract.clone(),
            &Cw721ExecuteMsg::TransferNft {
                recipient: env.contract.address.to_string(),
                token_id: shield.nft_id.to_string(),
            },
            vec![],
        )?;
        res = res.add_message(transfer_shield);
    }

    // Burn the gem_base, gem_materials and shield NFTs
    let burn_gem_base = wasm_execute(
        gem_base.nft_contract.clone(),
        &Cw721ExecuteMsg::Burn {
            token_id: gem_base.nft_id.to_string(),
        },
        vec![],
    )?;

    res = res.add_message(burn_gem_base);

    for gem_material in gem_materials {
        let burn_gem_material = wasm_execute(
            gem_material.nft_contract.clone(),
            &Cw721ExecuteMsg::Burn {
                token_id: gem_material.nft_id.to_string(),
            },
            vec![],
        )?;
        res = res.add_message(burn_gem_material);
    }

    if let Some(shield) = shield {
        let burn_shield = wasm_execute(
            shield.nft_contract.clone(),
            &Cw721ExecuteMsg::Burn {
                token_id: shield.nft_id.to_string(),
            },
            vec![],
        )?;
        res = res.add_message(burn_shield);
    }

    // Mint the new gem NFT from auragon_collection
    let mint_gem = wasm_execute(
        "auragon_collection".to_string(),
        &Cw721BaseExecuteMsg::Mint {
            recipient: info.sender.to_string(),
            token_id: "1".to_string(),
            name: "Gem".to_string(),
            description: "A gem".to_string(),
            image: None,
        },
        vec![],
    )?;

    Ok(res)
}

/// validate string if it is valid bench32 string addresss
fn addr_validate(api: &dyn Api, addr: &str) -> Result<Addr, ContractError> {
    let addr = api
        .addr_validate(addr)
        .map_err(|_| ContractError::InvalidAddress {})?;
    Ok(addr)
}
