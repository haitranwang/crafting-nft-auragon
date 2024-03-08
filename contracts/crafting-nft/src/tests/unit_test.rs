#[cfg(test)]
mod unit_tests {
    use std::str::FromStr;
    use crate::contract::{execute, instantiate};
    use crate::error::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg};
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        coins, to_json_binary, Addr, BlockInfo, Coin, ContractInfo, CosmosMsg, Env, OwnedDeps,
        Response, Timestamp, Uint128, WasmMsg,
    };
    use cw721_base::{ExecuteMsg as CW721ExecuteMsg, Extension as CW721Extension};

    const CREATOR: &str = "creator";
    const USER: &str = "user";
    const NOIS_PROXY: &str = "nois proxy";

    // SETUP ENVIROMENT
    fn default_setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        // instantiate a cw721 contract
        let msg = CW721ExecuteMsg::Instantiate {
            name: "test".to_string(),
            symbol: "TEST".to_string(),
            minter: String::from(CREATOR),
            meta: None,
            extension: Some(CW721Extension {
                contract_addr: Addr::unchecked("extension"),
                msg: to_json_binary(&CW721ExecuteMsg::Mint {
                    to: Addr::unchecked(USER),
                    token_id: "1".to_string(),
                    name: "name".to_string(),
                    description: Some("description".to_string()),
                    image: None,
                })
                .unwrap(),
            }),
        };


        let msg = InstantiateMsg {
            random_seed: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            nois_proxy: NOIS_PROXY.to_string(),
            dragon_collection: "dragon_collection".to_string(),
        };

        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        return deps;
    }
}