use std::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, has_coins, to_binary, to_json_binary, wasm_execute, Addr, Api, BalanceResponse, BankMsg, BankQuery, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Empty, Env, MessageInfo, Order, QueryRequest, Response, StdResult, Storage, Timestamp, Uint128, WasmMsg, WasmQuery
};
use cw2::set_contract_version;

use cw721::{Cw721ExecuteMsg, Cw721QueryMsg, NftInfoResponse};
use cw721_base::ExecuteMsg as Cw721BaseExecuteMsg;

use cw20::Cw20ExecuteMsg;
use nois::{randomness_from_str, select_from_weighted, sub_randomness_with_key, NoisCallback, ProxyExecuteMsg};

use crate::{error::ContractError, msg::{ExecuteMsg, InstantiateMsg, QueryMsg}, state::{AuragonURI, Config, GemInfo, GemMetadata, Metadata, RandomJob, RequestForgeGemInfo, Trait, UserInfo, AURAGON_LATEST_TOKEN_ID, AURAGON_URI, BLUE_GEM_WORK_POWER, CONFIG, CURRENT_QUEUE_ID, GEM_RATIO, GEM_WORK_LOAD, GOLD_GEM_WORK_POWER, RANDOM_JOBS, RANDOM_SEED, RED_GEM_WORK_POWER, SHIELD_LATEST_TOKEN_ID, SHIELD_URI, USERS_IN_QUEUE, WHITE_GEM_WORK_POWER}};


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
        dragon_collection: addr_validate(deps.api, &msg.dragon_collection)?,
        auragon_collection: addr_validate(deps.api, &msg.auragon_collection)?,
        shield_collection: addr_validate(deps.api, &msg.shield_collection)?,
    };
    CONFIG.save(deps.storage, &config)?;

    AURAGON_URI.save(
        deps.storage,
        &AuragonURI {
            white: msg.white_gem_uri,
            blue: msg.blue_gem_uri,
            gold: msg.gold_gem_uri,
            red: msg.red_gem_uri,
        },
    )?;

    SHIELD_URI.save(deps.storage, &msg.shield_uri)?;

    // save the init RANDOM_SEED to the storage
    let randomness = randomness_from_str(msg.random_seed).unwrap();
    RANDOM_SEED.save(deps.storage, &randomness)?;
    // save white gem work power to the storage
    WHITE_GEM_WORK_POWER.save(deps.storage, &msg.white_gem_work_power)?;
    // save blue gem work power to the storage
    BLUE_GEM_WORK_POWER.save(deps.storage, &msg.blue_gem_work_power)?;
    // save gold gem work power to the storage
    GOLD_GEM_WORK_POWER.save(deps.storage, &msg.gold_gem_work_power)?;
    // save red gem work power to the storage
    RED_GEM_WORK_POWER.save(deps.storage, &msg.red_gem_work_power)?;
    // save gem ratio to the storage
    GEM_RATIO.save(deps.storage, &msg.gem_ratio)?;
    // save gem work load to the storage
    GEM_WORK_LOAD.save(deps.storage, &msg.gem_work_load)?;
    // Initialize the token id
    AURAGON_LATEST_TOKEN_ID.save(deps.storage, &0)?;
    SHIELD_LATEST_TOKEN_ID.save(deps.storage, &0)?;

    // Initialize the current queue id
    CURRENT_QUEUE_ID.save(deps.storage, &0)?;

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
        ExecuteMsg::ForgeGem { request_forge_id, forge_gem_list }
            => execute_forge_gem(deps, env, info, request_forge_id, forge_gem_list),
        // ExecuteMsg::ForgeGemType1 { user_list } => execute_forge_gem_type_1(deps, env, info, user_list),
        //nois callback
        ExecuteMsg::NoisReceive { callback } => nois_receive(deps, env, info, callback),
        ExecuteMsg::UpdateCollection {
            dragon_collection,
            auragon_collection,
            shield_collection,
        } => update_collection(deps, env, info, dragon_collection, auragon_collection, shield_collection),
        ExecuteMsg::MintAuragonGem { owner, gem_trait
        } => mint_auragon_gem(deps, env, info, owner, gem_trait),
        ExecuteMsg::MintShieldGem { owner } => mint_shield_gem(deps, env, info, owner),
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
    let mut latest_token_id = AURAGON_LATEST_TOKEN_ID.load(deps.storage)?;

    // Load current queue id
    let mut current_queue_id = CURRENT_QUEUE_ID.load(deps.storage)?;

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
    let user_in_queue = UserInfo {
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
    request_forge_id: String,
    forge_gem_list: Vec<RequestForgeGemInfo>,
) -> Result<Response, ContractError> {
    // Load the config
    let config = CONFIG.load(deps.storage)?;
    // Load auragon uri
    let auragon_uri = AURAGON_URI.load(deps.storage)?;
    // Load the nois_proxy
    let nois_proxy = config.nois_proxy;

    // get dragon_collection
    let dragon_collection = config.dragon_collection;

    // Load the auragon_collection
    let auragon_collection = config.auragon_collection;

    // Load the shield_collection
    let shield_collection = config.shield_collection;

    // Get user address list from forge_gem_list
    let user_addr_list: Vec<Addr> = forge_gem_list.iter().map(|forge_gem| {
        forge_gem.user_addr.clone()
    }).collect();

    // Get gem_base list from forge_gem_list
    let gem_base_list: Vec<GemInfo> = forge_gem_list.iter().map(|forge_gem| {
        forge_gem.gem_base.clone()
    }).collect();

    // Check owner of gem_base should be the same as the user_addr
    for (user_addr, gem_base) in user_addr_list.iter().zip(gem_base_list.iter()) {
        let query_msg = Cw721QueryMsg::OwnerOf {
            token_id: gem_base.nft_id.clone(),
            include_expired: None,
        };

        let query_response: StdResult<cw721::OwnerOfResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: gem_base.nft_contract.to_string(),
            msg: to_json_binary(&query_msg).unwrap(),
        }));

        match query_response {
            Ok(response) => {
                if response.owner != user_addr.to_string() {
                    return Err(ContractError::Unauthorized {});
                }
            },
            Err(_) => return Err(ContractError::Unauthorized {}),
        }
    }

    // get gem_base nft_contract
    let gem_base_nft_contract_user_list: Vec<Addr> = forge_gem_list.iter().map(|forge_gem| {
        forge_gem.gem_base.nft_contract.clone()
    }).collect();
    // get gem_base_nft_id_user_list
    let gem_base_nft_id_user_list: Vec<String> = forge_gem_list.iter().map(|forge_gem| {
        forge_gem.gem_base.nft_id.clone()
    }).collect();

    // loop through gem_base_nft_contract_user_list and gem_base_nft_id_user_list and get token uri if contract is dragon_collection and get color and star if contract is auragon_collection
    let gem_base_nft_color_and_star_user_list: Vec<String>
        = gem_base_nft_contract_user_list.iter().zip(gem_base_nft_id_user_list.iter()).map(|(contract, id)| {
        if contract == &dragon_collection {
            let query_msg = Cw721QueryMsg::NftInfo { token_id: id.clone() };

            let query_response: StdResult<cw721::NftInfoResponse<Metadata>> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contract.to_string(),
                msg: to_json_binary(&query_msg).unwrap(),
            }));

            match query_response {
                Ok(response) => {
                    match response.token_uri.unwrap().as_str() {
                        "ipfs://Qme1dXSRNSqYvVQSDEmoL6WHMLqrYajZkszYhbRGj2F2oa" => "white-1".to_string(),
                        "ipfs://QmSp3iYpenTNr69g2EDSS128Vs1oRV2EHW8vakZ2Ro8G6P" => "blue-1".to_string(),
                        "ipfs://QmQP3N4jxJKGXPx18PgrjdhGLqYjX2qtinZ4q4YBeQhpw7" => "gold-1".to_string(),
                        "ipfs://QmTUy7E1UnLcbasfQap38kfxBAsFWzPfLmQimTh7pNw4QT" => "red-1".to_string(),
                        _ => "".to_string()
                    }
                },
                Err(_) => "".to_string()
            }
        } else {
            let query_msg = Cw721QueryMsg::NftInfo { token_id: id.clone() };

            let query_response: StdResult<cw721::NftInfoResponse<Metadata>> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contract.to_string(),
                msg: to_json_binary(&query_msg).unwrap(),
            }));

            match query_response {
                Ok(response) => {
                    match response.extension.attributes.as_ref().unwrap()[0].value.as_str() {
                        "white" => "white-".to_string() + &response.extension.attributes.unwrap()[1].value,
                        "blue" => "blue-".to_string() + &response.extension.attributes.unwrap()[1].value,
                        "gold" => "gold-".to_string() + &response.extension.attributes.unwrap()[1].value,
                        "red" => "red-".to_string() + &response.extension.attributes.unwrap()[1].value,
                        _ => "".to_string()
                    }
                },
                Err(_) => "".to_string()
            }
        }
    }).collect();

    // Get gem_materials list from forge_gem_list
    let gem_materials_list: Vec<Vec<GemInfo>> = forge_gem_list.iter().map(|forge_gem| {
        forge_gem.gem_materials.clone()
    }).collect();

    // Check owner of gem_materials should be the same as the user_addr
    for (user_addr, gem_materials) in user_addr_list.iter().zip(gem_materials_list.iter()) {
        for gem_material in gem_materials {
            let query_msg = Cw721QueryMsg::OwnerOf {
                token_id: gem_material.nft_id.clone(),
                include_expired: None,
            };

            let query_response: StdResult<cw721::OwnerOfResponse> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: gem_material.nft_contract.to_string(),
                msg: to_json_binary(&query_msg).unwrap(),
            }));

            match query_response {
                Ok(response) => {
                    if response.owner != user_addr.to_string() {
                        return Err(ContractError::Unauthorized {});
                    }
                },
                Err(_) => return Err(ContractError::Unauthorized {}),
            }
        }
    }

    // Get shield_id list from forge_gem_list
    let shield_id_list: Vec<Option<String>> = forge_gem_list.iter().map(|forge_gem| {
        forge_gem.shield_id.clone()
    }).collect();

    // Check owner of shield_id should be the same as the user_addr
    for (user_addr, shield_id) in user_addr_list.iter().zip(shield_id_list.iter()) {
        if let Some(ref shield_id) = shield_id {
            let query_msg = Cw721QueryMsg::OwnerOf {
                token_id: shield_id.clone(),
                include_expired: None,
            };

            let query_response: StdResult<cw721::OwnerOfResponse> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: shield_collection.to_string(),
                msg: to_json_binary(&query_msg).unwrap(),
            }));

            match query_response {
                Ok(response) => {
                    if response.owner != user_addr.to_string() {
                        return Err(ContractError::Unauthorized {});
                    }
                },
                Err(_) => return Err(ContractError::Unauthorized {}),
            }
        }
    }

    let funds = info.funds;

    let mut res = Response::new();

    // Get user_user_success_rate_list from forge_gem_list
    let user_success_rate_list: Vec<(Addr, u32)> = forge_gem_list.iter().map(|forge_gem| {
        (forge_gem.user_addr.clone(), forge_gem.success_rate.parse::<u32>().unwrap())
    }).collect();

    // Make randomness request message to NOIS proxy contract
    let msg_make_randomess = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: nois_proxy.into(),
        msg: to_json_binary(&ProxyExecuteMsg::GetNextRandomness {
            job_id: request_forge_id.clone(),
        })?,
        funds,
    });

    res = res.add_message(msg_make_randomess);

    // save job for mapping callback response to request
    let random_job = RandomJob {
        gem_base_nft_color_and_star_user_list,
        user_success_rate_list,
        timestamp: env.block.time,
    };

    RANDOM_JOBS.save(deps.storage, request_forge_id.clone(), &random_job)?;
    Ok(res)
}

// pub fn execute_forge_gem_type_1(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     user_list: Vec<UserInfo>,
// ) -> Result<Response, ContractError> {
//     // Load the config
//     let config = CONFIG.load(deps.storage)?;
//     // Load auragon uri
//     let auragon_uri = AURAGON_URI.load(deps.storage)?;
//     // Load the nois_proxy
//     let nois_proxy = config.nois_proxy;

//     // Load the auragon_collection
//     let auragon_collection = config.auragon_collection;

//     // Load the shield_collection
//     let shield_collection = config.shield_collection;

//     // Load the latest token id
//     let mut latest_token_id = AURAGON_LATEST_TOKEN_ID.load(deps.storage)?;

//     let job_id = format!("{}/{}", env.block.time, info.sender);

//     let mut funds = info.funds;

//     let mut res = Response::new();

//     // A list of user and their success rate
//     let user_success_rate_list: Vec<(Addr, u32)> = convert_to_user_success_rate_list(&deps, user_list.clone());

//     // Make randomness request message to NOIS proxy contract
//     let msg_make_randomess = CosmosMsg::Wasm(WasmMsg::Execute {
//         contract_addr: nois_proxy.into(),
//         msg: to_json_binary(&ProxyExecuteMsg::GetNextRandomness {
//             job_id: job_id.clone(),
//         })?,
//         funds,
//     });

//     res = res.add_message(msg_make_randomess);

//     // save job for mapping callback response to request
//     let random_job = RandomJob {
//         user_success_rate_list,
//         timestamp: env.block.time,
//     };

//     RANDOM_JOBS.save(deps.storage, job_id.clone(), &random_job)?;

//     // Loop through user_list and forge the gem
//     for user in user_list {
//         // Mint the new gem NFT
//         let gem_trait = GemMetadata {
//             color: "red".to_string(),
//             star: 1,
//         };

//         let extension = Metadata {
//             attributes: vec![
//                 Trait {
//                     display_type: None,
//                     trait_type: "color".to_string(),
//                     value: gem_trait.color.clone(),
//                 },
//                 Trait {
//                     display_type: None,
//                     trait_type: "star".to_string(),
//                     value: gem_trait.star.to_string(),
//                 }
//             ].into(),
//             ..Default::default()
//         };

//         let token_uri = match gem_trait.color.as_str() {
//             "white" => auragon_uri.white[gem_trait.star as usize].clone(),
//             "blue" => auragon_uri.blue[gem_trait.star as usize].clone(),
//             "gold" => auragon_uri.gold[gem_trait.star as usize].clone(),
//             "red" => auragon_uri.red[gem_trait.star as usize].clone(),
//             _ => "".to_string(),
//         };

//         // Mint the new gem NFT from auragon_collection with token id increment by 1
//         latest_token_id += 1;

//         // Mint the new gem NFT from auragon_collection
//         let mint_gem = wasm_execute(
//             auragon_collection.to_string(),
//             &Cw721BaseExecuteMsg::Mint::<Metadata, Empty> {
//                 token_id: latest_token_id.to_string(),
//                 owner: user.user_addr.to_string(),
//                 token_uri: Some(token_uri),
//                 extension,
//             },
//             vec![],
//         )?;

//         // Update the latest token id
//         AURAGON_LATEST_TOKEN_ID.save(deps.storage, &latest_token_id)?;

//         res = res.add_message(mint_gem);
//     }
//     Ok(res)
// }

pub fn update_collection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    dragon_collection: Option<String>,
    auragon_collection: Option<String>,
    shield_collection: Option<String>,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    // ensure_eq!(
    //     info.sender,
    //     config.nois_proxy,
    //     ContractError::Unauthorized {}
    // );

    if let Some(ref dragon_collection) = dragon_collection {
        CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
            config.dragon_collection = addr_validate(deps.api, &dragon_collection)?;
            Ok(config)
        })?;
    }

    if let Some(ref auragon_collection) = auragon_collection {
        CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
            config.auragon_collection = addr_validate(deps.api, &auragon_collection)?;
            Ok(config)
        })?;
    }

    if let Some(ref shield_collection) = shield_collection {
        CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
            config.shield_collection = addr_validate(deps.api, &shield_collection)?;
            Ok(config)
        })?;
    }

    Ok(Response::new()
        .add_attribute("action", "update_collection")
        .add_attribute("dragon_collection", dragon_collection.unwrap_or_default())
        .add_attribute("auragon_collection", auragon_collection.unwrap_or_default())
        .add_attribute("shield_collection", shield_collection.unwrap_or_default()))
}

pub fn mint_auragon_gem(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    gem_trait: GemMetadata,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    // Load the latest token id
    let mut latest_token_id = AURAGON_LATEST_TOKEN_ID.load(deps.storage)?;
    // Load auragon uri
    let auragon_uri = AURAGON_URI.load(deps.storage)?;

    // ensure_eq!(
    //     info.sender,
    //     config.nois_proxy,
    //     ContractError::Unauthorized {}
    // );

    let auragon_collection = config.auragon_collection;

    // Mint the new gem NFT from auragon_collection with token id increment by 1
    latest_token_id += 1;

    let extension = Metadata {
        attributes: vec![
            Trait {
                display_type: None,
                trait_type: "color".to_string(),
                value: gem_trait.color.clone(),
            },
            Trait {
                display_type: None,
                trait_type: "star".to_string(),
                value: gem_trait.star.to_string(),
            }
        ].into(),
        ..Default::default()
    };

    let token_uri = match gem_trait.color.as_str() {
        "white" => auragon_uri.white[gem_trait.star as usize].clone(),
        "blue" => auragon_uri.blue[gem_trait.star as usize].clone(),
        "gold" => auragon_uri.gold[gem_trait.star as usize].clone(),
        "red" => auragon_uri.red[gem_trait.star as usize].clone(),
        _ => "".to_string(),
    };

    // Mint the new gem NFT from auragon_collection
    let mint_gem = wasm_execute(
        auragon_collection.to_string(),
        &Cw721BaseExecuteMsg::Mint::<Metadata, Empty> {
            token_id: latest_token_id.to_string(),
            owner: owner.to_string(),
            token_uri: Some(token_uri),
            extension,
        },
        vec![],
    )?;

    // Update the latest token id
    AURAGON_LATEST_TOKEN_ID.save(deps.storage, &latest_token_id)?;

    Ok(Response::new()
        .add_message(mint_gem)
        .add_attribute("action", "mint_auragon_gem")
        .add_attribute("token_id", &latest_token_id.to_string())
        .add_attribute("owner", owner))
}

pub fn mint_shield_gem(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    // Load the latest token id
    let mut latest_token_id = SHIELD_LATEST_TOKEN_ID.load(deps.storage)?;
    // Load shield uri
    let shield_uri = SHIELD_URI.load(deps.storage)?;

    // ensure_eq!(
    //     info.sender,
    //     config.nois_proxy,
    //     ContractError::Unauthorized {}
    // );

    let shield_collection = config.shield_collection;

    // Mint the new gem NFT from shield_collection with token id increment by 1
    latest_token_id += 1;

    // Mint the new gem NFT from shield_collection
    let mint_gem = wasm_execute(
        shield_collection.to_string(),
        &Cw721BaseExecuteMsg::Mint::<Metadata, Empty> {
            token_id: latest_token_id.to_string(),
            owner: owner.to_string(),
            token_uri: Some(shield_uri),
            extension: Default::default(),
        },
        vec![],
    )?;

    // Update the latest token id
    SHIELD_LATEST_TOKEN_ID.save(deps.storage, &latest_token_id)?;

    Ok(Response::new()
        .add_message(mint_gem)
        .add_attribute("action", "mint_shield_gem")
        .add_attribute("token_id", &latest_token_id.to_string())
        .add_attribute("owner", owner))
}

pub fn nois_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    callback: NoisCallback,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    ensure_eq!(
        info.sender,
        config.nois_proxy,
        ContractError::Unauthorized {}
    );

    let job_id = callback.job_id;
    let randomness: [u8; 32] = callback
        .randomness
        .to_array()
        .map_err(|_| ContractError::InvalidRandomness {})?;

    let random_job: RandomJob =
        if let Some(job) = RANDOM_JOBS.may_load(deps.storage, job_id.clone())? {
            job
        } else {
            return Err(ContractError::RandomJobNotFound {});
        };

    // init a key for the random provider from the job id
    let key = format!("{}", job_id.clone());

    let res = select_gem_rewards(
        deps.storage,
        randomness,
        &key,
        random_job.timestamp,
    )?;

    // job finished, just remove
    RANDOM_JOBS.remove(deps.storage, job_id.clone());

    Ok(res
        .add_attribute("action", "nois_receive")
        .add_attribute("job_id", job_id))
}

fn select_gem_rewards(
    storage: &mut dyn Storage,
    random_seed: [u8; 32],
    key: &String,
    forges: Timestamp,
) -> Result<Response, ContractError> {
    // update random seed
    RANDOM_SEED.save(storage, &random_seed)?;
    // load config
    let config: Config = CONFIG.load(storage)?;
    // load random job
    let random_job: RandomJob = RANDOM_JOBS.load(storage, key.clone())?;
    // load user success rate list
    let user_success_rate_list = random_job.user_success_rate_list;
    // load gem base nft color and star user list
    let mut gem_base_nft_color_and_star_user_list = random_job.gem_base_nft_color_and_star_user_list;
    let mut res = Response::new();
    let mut latest_token_id = AURAGON_LATEST_TOKEN_ID.load(storage)?;
    // loop through user_success_rate_list and select gem rewards with select_from_weighted
    for (user_addr, success_rate) in user_success_rate_list {
        let weights_list: Vec<(&str, u32)>;
        // make a new vec of success rate for each user by sub with 100
        let failure_rate = 100 - success_rate;
        if failure_rate == 0 {
            weights_list = vec![("success", success_rate)];
        } else {
            weights_list = vec![("success", success_rate), ("failure", failure_rate)];
        }
        // select from weighted
        let selected = select_from_weighted(random_seed, &weights_list).unwrap();
        // if selected is success, mint the new gem NFT with color and star = star + 1 from gem_base_nft_color_and_star_user_list
        if selected == "success" {
            let color_and_star = gem_base_nft_color_and_star_user_list.pop().unwrap();
            let color_and_star_split: Vec<&str> = color_and_star.split("-").collect();
            let color = color_and_star_split[0];
            let star = color_and_star_split[1].parse::<u32>().unwrap();
            let auragon_collection = config.auragon_collection.clone();
            // increase the latest token id by 1
            latest_token_id += 1;
            let token_uri = match color {
                "white" => AURAGON_URI.load(storage)?.white[star as usize].clone(),
                "blue" => AURAGON_URI.load(storage)?.blue[star as usize].clone(),
                "gold" => AURAGON_URI.load(storage)?.gold[star as usize].clone(),
                "red" => AURAGON_URI.load(storage)?.red[star as usize].clone(),
                _ => "".to_string(),
            };
            let extension = Metadata {
                attributes: vec![
                    Trait {
                        display_type: None,
                        trait_type: "color".to_string(),
                        value: color.to_string(),
                    },
                    Trait {
                        display_type: None,
                        trait_type: "star".to_string(),
                        value: (star + 1).to_string(),
                    }
                ].into(),
                ..Default::default()
            };
            let mint_gem = wasm_execute(
                auragon_collection.to_string(),
                &Cw721BaseExecuteMsg::Mint::<Metadata, Empty> {
                    token_id: (latest_token_id).to_string(),
                    owner: user_addr.to_string(),
                    token_uri: Some(token_uri),
                    extension,
                },
                vec![],
            )?;
            // add message mint_gem and approve_all to the response
            res = res.add_message(mint_gem);
            // add attribute to the response user success and token id minted
            res = res.add_attribute("user_success", user_addr.to_string())
                .add_attribute("new_token_id", (latest_token_id).to_string());
        } else {
            // add attribute to the response user failure
            res = res.add_attribute("user_failure", user_addr.to_string());
        }
    }
    // update the latest token id
    AURAGON_LATEST_TOKEN_ID.save(storage, &(latest_token_id))?;
    Ok(res.add_attribute("action", "select_gem_rewards DONEEEEEEE"))
}

// fn convert_to_user_success_rate_list(deps: &DepsMut, user_list: Vec<UserInfo>) -> Vec<(Addr, u32)> {
//     // get CONFIG
//     let config: Config = CONFIG.load(deps.storage).unwrap();
//     // get white gem work power
//     let white_gem_work_power = WHITE_GEM_WORK_POWER.load(deps.storage).unwrap();
//     // get blue gem work power
//     let blue_gem_work_power = BLUE_GEM_WORK_POWER.load(deps.storage).unwrap();
//     // get gold gem work power
//     let gold_gem_work_power = GOLD_GEM_WORK_POWER.load(deps.storage).unwrap();
//     // get red gem work power
//     let red_gem_work_power = RED_GEM_WORK_POWER.load(deps.storage).unwrap();
//     // get gem ratio
//     let gem_ratio = GEM_RATIO.load(deps.storage).unwrap();
//     // get gem work load from n star to n+1 star
//     let gem_work_load = GEM_WORK_LOAD.load(deps.storage).unwrap();
//     // get dragon_collection
//     let dragon_collection = config.dragon_collection;
//     // get gem_base nft_id
//     let gem_base_nft_id_user_list: Vec<String> = user_list.iter().map(|user| user.gem_base.nft_id.clone()).collect();
//     // get gem_base nft_contract
//     let gem_base_nft_contract_user_list: Vec<Addr> = user_list.iter().map(|user| user.gem_base.nft_contract.clone()).collect();
//     // get gem_materials nft_id
//     let gem_materials_nft_id_user_list: Vec<Vec<String>> = user_list.iter().map(|user| user.gem_materials.iter().map(|gem| gem.nft_id.clone()).collect()).collect();
//     // get gem_materials nft_contract
//     let gem_materials_nft_contract_user_list: Vec<Vec<Addr>> = user_list.iter().map(|user| user.gem_materials.iter().map(|gem| gem.nft_contract.clone()).collect()).collect();
//     // loop through gem_base_nft_contract_user_list and gem_base_nft_id_user_list and get token uri if contract is dragon_collection and get color and star if contract is auragon_collection
//     let gem_base_nft_color_and_star_user_list: Vec<String>
//         = gem_base_nft_contract_user_list.iter().zip(gem_base_nft_id_user_list.iter()).map(|(contract, id)| {
//         if contract == &dragon_collection {
//             let query_msg = Cw721QueryMsg::NftInfo { token_id: id.clone() };

//             let query_response: StdResult<cw721::NftInfoResponse<Metadata>> =
//             deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
//                 contract_addr: contract.to_string(),
//                 msg: to_json_binary(&query_msg).unwrap(),
//             }));

//             match query_response {
//                 Ok(response) => {
//                     match response.token_uri.unwrap().as_str() {
//                         "ipfs://Qme1dXSRNSqYvVQSDEmoL6WHMLqrYajZkszYhbRGj2F2oa" => "white-1".to_string(),
//                         "ipfs://QmSp3iYpenTNr69g2EDSS128Vs1oRV2EHW8vakZ2Ro8G6P" => "blue-1".to_string(),
//                         "ipfs://QmQP3N4jxJKGXPx18PgrjdhGLqYjX2qtinZ4q4YBeQhpw7" => "gold-1".to_string(),
//                         "ipfs://QmTUy7E1UnLcbasfQap38kfxBAsFWzPfLmQimTh7pNw4QT" => "red-1".to_string(),
//                         _ => "".to_string()
//                     }
//                 },
//                 Err(_) => "".to_string()
//             }
//         } else {
//             let query_msg = Cw721QueryMsg::NftInfo { token_id: id.clone() };

//             let query_response: StdResult<cw721::NftInfoResponse<Metadata>> =
//             deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
//                 contract_addr: contract.to_string(),
//                 msg: to_json_binary(&query_msg).unwrap(),
//             }));

//             match query_response {
//                 Ok(response) => {
//                     match response.extension.attributes.as_ref().unwrap()[0].value.as_str() {
//                         "white" => "white-".to_string() + &response.extension.attributes.unwrap()[1].value,
//                         "blue" => "blue-".to_string() + &response.extension.attributes.unwrap()[1].value,
//                         "gold" => "gold-".to_string() + &response.extension.attributes.unwrap()[1].value,
//                         "red" => "red-".to_string() + &response.extension.attributes.unwrap()[1].value,
//                         _ => "".to_string()
//                     }
//                 },
//                 Err(_) => "".to_string()
//             }
//         }
//     }).collect();
//     // loop through gem_materials_nft_contract_user_list and gem_materials_nft_id_user_list and get token uri if contract is dragon_collection and get color and star if contract is auragon_collection
//     let gem_materials_nft_color_and_star_user_list: Vec<Vec<String>>
//         = gem_materials_nft_contract_user_list.iter().zip(gem_materials_nft_id_user_list.iter()).map(|(contract, id)| {
//         contract.iter().zip(id.iter()).map(|(contract, id)| {
//             if contract == &dragon_collection {
//                 let query_msg = Cw721QueryMsg::NftInfo { token_id: id.clone() };

//                 let query_response: StdResult<cw721::NftInfoResponse<Metadata>> =
//                 deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
//                     contract_addr: contract.to_string(),
//                     msg: to_json_binary(&query_msg).unwrap(),
//                 }));

//                 match query_response {
//                     Ok(response) => {
//                         match response.token_uri.unwrap().as_str() {
//                             "ipfs://Qme1dXSRNSqYvVQSDEmoL6WHMLqrYajZkszYhbRGj2F2oa" => "white-1".to_string(),
//                             "ipfs://QmSp3iYpenTNr69g2EDSS128Vs1oRV2EHW8vakZ2Ro8G6P" => "blue-1".to_string(),
//                             "ipfs://QmQP3N4jxJKGXPx18PgrjdhGLqYjX2qtinZ4q4YBeQhpw7" => "gold-1".to_string(),
//                             "ipfs://QmTUy7E1UnLcbasfQap38kfxBAsFWzPfLmQimTh7pNw4QT" => "red-1".to_string(),
//                             _ => "".to_string()
//                         }
//                     },
//                     Err(_) => "".to_string()
//                 }
//             } else {
//                 let query_msg = Cw721QueryMsg::NftInfo { token_id: id.clone() };

//                 let query_response: StdResult<cw721::NftInfoResponse<Metadata>> =
//                 deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
//                     contract_addr: contract.to_string(),
//                     msg: to_json_binary(&query_msg).unwrap(),
//                 }));

//                 match query_response {
//                     Ok(response) => {
//                         match response.extension.attributes.as_ref().unwrap()[0].value.as_str() {
//                             "white" => "white-".to_string() + &response.extension.attributes.unwrap()[1].value,
//                             "blue" => "blue-".to_string() + &response.extension.attributes.unwrap()[1].value,
//                             "gold" => "gold-".to_string() + &response.extension.attributes.unwrap()[1].value,
//                             "red" => "red-".to_string() + &response.extension.attributes.unwrap()[1].value,
//                             _ => "".to_string()
//                         }
//                     },
//                     Err(_) => "".to_string()
//                 }
//             }
//         }).collect()
//     }).collect();

//     // convert gem_materials_nft_color_and_star_user_list to gem_materials_work_power_user_list base on WHITE_GEM_WORK_POWER, BLUE_GEM_WORK_POWER, GOLD_GEM_WORK_POWER, RED_GEM_WORK_POWER
//     // take the star from color-star string and convert it to u8
//     let gem_materials_work_power_user_list: Vec<Vec<Decimal>> = gem_materials_nft_color_and_star_user_list.iter().map(|gem_materials| {
//         gem_materials.iter().map(|gem_material| {
//             let star: u8 = gem_material.split("-").collect::<Vec<&str>>()[1].parse().unwrap();
//             match gem_material.split("-").collect::<Vec<&str>>()[0] {
//                 "white" => white_gem_work_power[star as usize - 1],
//                 "blue" => blue_gem_work_power[star as usize - 1],
//                 "gold" => gold_gem_work_power[star as usize - 1],
//                 "red" => red_gem_work_power[star as usize - 1],
//                 _ => Decimal::zero()
//             }
//         }).collect()
//     }).collect();

//     // convert gem_materials_work_power_user_list to gem_materials_work_power_user_list_sum
//     let gem_materials_work_power_user_list_sum: Vec<Decimal> = gem_materials_work_power_user_list.iter().map(|gem_materials| {
//         gem_materials.iter().sum()
//     }).collect();

//     // calculate the user work load base on star of gem_base
//     let user_work_load: Vec<Decimal> = gem_base_nft_color_and_star_user_list.iter().map(|gem_base| {
//         let star: u8 = gem_base.split("-").collect::<Vec<&str>>()[1].parse().unwrap();
//         gem_work_load[star as usize - 1]
//     }).collect();

//     // calculate the user success rate base on user work load and gem_materials_work_power_user_list_sum (success_rate = gem_materials_work_power_user_list_sum / user_work_load)
//     let user_success_rate: Vec<Decimal> = user_work_load.iter().zip(gem_materials_work_power_user_list_sum.iter()).map(|(work_load, work_power)| {
//         work_power / work_load
//     }).collect();

//     // convert user_addr and user_success_rate to Vec<(Addr, u32)>
//     let user_success_rate_list: Vec<(Addr, u32)> = user_list.iter().map(|user| {
//         (user.user_addr.clone(), user_success_rate[user_list.iter().position(|x| x == user).unwrap()].to_u32().unwrap())
//     }).collect();

//     user_success_rate_list
// }

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::RandomSeed {} => to_json_binary(&query_random_seed(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

fn query_random_seed(deps: Deps) -> StdResult<[u8; 32]> {
    RANDOM_SEED.load(deps.storage)
}

/// validate string if it is valid bench32 string addresss
fn addr_validate(api: &dyn Api, addr: &str) -> Result<Addr, ContractError> {
    let addr = api
        .addr_validate(addr)
        .map_err(|_| ContractError::InvalidAddress {})?;
    Ok(addr)
}

// Unit test for select_gem_rewards
#[cfg(test)]
mod test_select_gem_rewards {
    use cosmwasm_std::{testing::mock_dependencies, Addr, Timestamp};

    use crate::{contract::select_gem_rewards, state::{AuragonURI, Config, RandomJob, AURAGON_LATEST_TOKEN_ID, AURAGON_URI, CONFIG, RANDOM_JOBS}};

    #[test]
    fn test_select_gem_rewards() {
        let mut deps = mock_dependencies();
        let config = {
            Config {
                nois_proxy: Addr::unchecked("nois_proxy"),
                auragon_collection: Addr::unchecked("auragon_collection"),
                shield_collection: Addr::unchecked("shield_collection"),
                dragon_collection: Addr::unchecked("dragon_collection"),
            }
        };
        CONFIG.save(&mut deps.storage, &config).unwrap();
        let auragon_gem_latest_token_id = 1;
        AURAGON_LATEST_TOKEN_ID.save(&mut deps.storage, &auragon_gem_latest_token_id).unwrap();
        let auragon_uri = AuragonURI {
            white: ["ipfs://1".to_string(), "ipfs://2".to_string(), "ipfs://3".to_string(), "ipfs://4".to_string(), "ipfs://5".to_string(), "ipfs://6".to_string(), "ipfs://7".to_string()],
            blue: ["ipfs://1".to_string(), "ipfs://2".to_string(), "ipfs://3".to_string(), "ipfs://4".to_string(), "ipfs://5".to_string(), "ipfs://6".to_string(), "ipfs://7".to_string()],
            gold: ["ipfs://1".to_string(), "ipfs://2".to_string(), "ipfs://3".to_string(), "ipfs://4".to_string(), "ipfs://5".to_string(), "ipfs://6".to_string(), "ipfs://7".to_string()],
            red: ["ipfs://1".to_string(), "ipfs://2".to_string(), "ipfs://3".to_string(), "ipfs://4".to_string(), "ipfs://5".to_string(), "ipfs://6".to_string(), "ipfs://7".to_string()],
        };
        AURAGON_URI.save(&mut deps.storage, &auragon_uri).unwrap();

        let random_job = RandomJob {
            gem_base_nft_color_and_star_user_list: vec!["white-1".to_string(), "blue-1".to_string(), "gold-1".to_string(), "red-1".to_string()],
            user_success_rate_list: vec![(Addr::unchecked("addr1"), 100), (Addr::unchecked("addr2"), 100), (Addr::unchecked("addr3"), 100), (Addr::unchecked("addr4"), 100)],
            timestamp: Timestamp::from_seconds(0),
        };
        let key = "1".to_string();

        RANDOM_JOBS.save(&mut deps.storage, key.clone(), &random_job).unwrap();
        let random_seed: [u8; 32] = [231, 176, 72, 156, 81, 254, 186, 90, 6, 217, 100, 59, 104, 255, 174, 43, 10, 192, 5, 213, 175, 182, 53, 224, 165, 219, 23, 212, 104, 217, 54, 105];
        let forges = Timestamp::from_seconds(0);
        let res = select_gem_rewards(&mut deps.storage, random_seed, &key, forges);
        // print AURA LATEST TOKEN ID
        let auragon_gem_latest_token_id = AURAGON_LATEST_TOKEN_ID.load(&deps.storage).unwrap();
        println!("AURA LATEST TOKEN: {:?}", auragon_gem_latest_token_id);
        assert_eq!(res.unwrap().messages, vec![]);
    }
}
