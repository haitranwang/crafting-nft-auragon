#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, has_coins, to_json_binary, wasm_execute, Addr, Api, BalanceResponse, BankMsg, BankQuery, Binary, Coin, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Order, QueryRequest, Response, StdResult, Storage, Timestamp, Uint128, WasmMsg
};
use cw2::set_contract_version;

use cw721::Cw721ExecuteMsg;
use cw721_base::ExecuteMsg as Cw721BaseExecuteMsg;

use cw20::Cw20ExecuteMsg;
use nois::randomness_from_str;

use crate::{error::ContractError, msg::{ExecuteMsg, InstantiateMsg, QueryMsg}, state::{Config, GemInfo, GemMetadata, UsersInQueue, CONFIG, LATEST_TOKEN_ID, RANDOM_SEED, USERS_IN_QUEUE}};


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
        nois_proxy,
        auragon_collection: addr_validate(deps.api, &msg.auragon_collection)?,
        shield_collection: addr_validate(deps.api, &msg.shield_collection)?,
    };
    CONFIG.save(deps.storage, &config)?;

    // save the init RANDOM_SEED to the storage
    let randomness = randomness_from_str(msg.random_seed).unwrap();
    RANDOM_SEED.save(deps.storage, &randomness)?;

    // Initialize the token id
    LATEST_TOKEN_ID.save(deps.storage, &0)?;

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
        ExecuteMsg::JoinQueue {
            gem_base,
            gem_materials,
            shield_id,
        } => execute_join_queue(deps, env, info, gem_base, gem_materials, shield_id),
        ExecuteMsg::ForgeGem { is_success } => execute_forge_gem(deps, env, info, is_success),
    }
}

pub fn execute_join_queue(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    gem_base: GemInfo,
    gem_materials: Vec<GemInfo>,
    shield_id: Option<String>,
) -> Result<Response, ContractError> {

    // Load the config
    let config = CONFIG.load(deps.storage)?;

    // Load the auragon_collection
    let auragon_collection = config.auragon_collection;

    // Load the shield_collection
    let shield_collection = config.shield_collection;

    // Load the latest token id
    let mut latest_token_id = LATEST_TOKEN_ID.load(deps.storage)?;

    let mut res = Response::new();
    // Approve the gem_base NFT
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

    // Approve the gem_materials NFTs
    for gem_material in &gem_materials {
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

    // Approve, Transfer the shield NFT to this contract and burn it
    if let Some(ref shield_id) = shield_id {
        let approve_shield = wasm_execute(
            shield_collection.clone(),
            &Cw721ExecuteMsg::Approve {
                spender: env.contract.address.to_string(),
                token_id: shield_id.to_string(),
                expires: None,
            },
            vec![],
        )?;
        res = res.add_message(approve_shield);

        let transfer_shield = wasm_execute(
            shield_collection.clone(),
            &Cw721ExecuteMsg::TransferNft {
                recipient: env.contract.address.to_string(),
                token_id: shield_id.to_string(),
            },
            vec![],
        )?;
        res = res.add_message(transfer_shield);

        let burn_shield = wasm_execute(
            shield_collection.clone(),
            &Cw721ExecuteMsg::Burn {
                token_id: shield_id.to_string(),
            },
            vec![],
        )?;

        res = res.add_message(burn_shield);
    }

    // Transfer the gem_base NFT to this contract
    let transfer_gem_base = wasm_execute(
        gem_base.nft_contract.clone(),
        &Cw721ExecuteMsg::TransferNft {
            recipient: env.contract.address.to_string(),
            token_id: gem_base.nft_id.to_string(),
        },
        vec![],
    )?;

    res = res.add_message(transfer_gem_base);

    // Transfer the gem_materials NFTs to this contract
    for gem_material in &gem_materials {
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

    // Add the user to the queue
    let user_in_queue = UsersInQueue {
        user_addr: info.sender.clone(),
        gem_base: gem_base.clone(),
        gem_materials: gem_materials.clone(),
        shield_id,
        timestamp: env.block.time,
    };

    // Add the user to the queue
    USERS_IN_QUEUE.push_back(deps.storage, &user_in_queue)?;

    // // Burn the gem_base, gem_materials NFTs
    // let burn_gem_base = wasm_execute(
    //     gem_base.nft_contract.clone(),
    //     &Cw721ExecuteMsg::Burn {
    //         token_id: gem_base.nft_id.to_string(),
    //     },
    //     vec![],
    // )?;

    // res = res.add_message(burn_gem_base);

    // Burn the gem_materials NFTs
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

    // // Mint the new gem NFT
    // let extension = GemMetadata {
    //     color: "red".to_string(),
    //     level: 1,
    // };

    // // Mint the new gem NFT from auragon_collection with token id increment by 1
    // latest_token_id += 1;

    // // Mint the new gem NFT from auragon_collection
    // let mint_gem = wasm_execute(
    //     auragon_collection.to_string(),
    //     &Cw721BaseExecuteMsg::Mint::<GemMetadata, Empty> {
    //         token_id: latest_token_id.to_string(),
    //         owner: info.sender.to_string(),
    //         token_uri: None,
    //         extension,
    //     },
    //     vec![],
    // )?;

    // res = res.add_message(mint_gem);

    // Update the latest token id
    // LATEST_TOKEN_ID.save(deps.storage, &latest_token_id)?;

    Ok(res.add_attributes(vec![
        ("action", "join_queue"),
        ("user", info.sender.as_str()),
    ]))
}

pub fn execute_forge_gem(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    is_success: bool,
) -> Result<Response, ContractError> {
    // Load the config
    let config = CONFIG.load(deps.storage)?;

    // Load the nois_proxy
    let nois_proxy = config.nois_proxy;

    // Load the auragon_collection
    let auragon_collection = config.auragon_collection;

    // Load the shield_collection
    let shield_collection = config.shield_collection;

    // Load the latest token id
    let mut latest_token_id = LATEST_TOKEN_ID.load(deps.storage)?;

    // Load the random seed
    let random_seed = RANDOM_SEED.load(deps.storage)?;

    let mut res = Response::new();

    // Loop through the queue and forge the gem
    let queue_len = USERS_IN_QUEUE.len(deps.storage)?;

    if queue_len == 0 {
        return Err(ContractError::PlayerNotFound { });
    }

    for _ in 0..queue_len {
        let user_in_queue = USERS_IN_QUEUE.pop_front(deps.storage)?;
        if is_success {
            // Mint the new gem NFT
            let extension = GemMetadata {
                color: "red".to_string(),
                level: 1,
            };

            // Mint the new gem NFT from auragon_collection with token id increment by 1
            latest_token_id += 1;

            // Mint the new gem NFT from auragon_collection
            let mint_gem = wasm_execute(
                auragon_collection.to_string(),
                &Cw721BaseExecuteMsg::Mint::<GemMetadata, Empty> {
                    token_id: latest_token_id.to_string(),
                    owner: user_in_queue.unwrap().user_addr.to_string(),
                    token_uri: None,
                    extension,
                },
                vec![],
            )?;

            res = res.add_message(mint_gem);
        }
    }
    Ok(res)
}
/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

/// validate string if it is valid bench32 string addresss
fn addr_validate(api: &dyn Api, addr: &str) -> Result<Addr, ContractError> {
    let addr = api
        .addr_validate(addr)
        .map_err(|_| ContractError::InvalidAddress {})?;
    Ok(addr)
}
