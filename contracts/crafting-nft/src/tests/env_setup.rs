#[cfg(test)]
pub mod env {
    use std::str::FromStr;

    use cosmwasm_std::{Addr, Coin, Decimal, Empty, Uint128};
    use cw20::MinterResponse;
    use cw721::Cw721ExecuteMsg;
    use cw721_base::{InstantiateMsg as Cw721InstantiateMsg, ExecuteMsg as Cw721BaseExecuteMsg, QueryMsg as Cw721BaseQueryMsg};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::contract::{
        execute as ForgingGemExecute, instantiate as ForgingGemInstantiate, query as ForgingGemQuery,
    };
    use crate::msg::{InstantiateMsg as ForgingGemInstantiateMsg, ExecuteMsg as ForgingGemExecuteMsg, QueryMsg as ForgingGemQueryMsg};

    pub const ADMIN: &str = "aura1000000000000000000000000000000000admin";
    pub const USER_1: &str = "aura1000000000000000000000000000000000user1";

    pub const NATIVE_DENOM: &str = "uaura";
    pub const NATIVE_BALANCE: u128 = 1_000_000_000_000u128;

    pub const NATIVE_DENOM_2: &str = "utaura";
    pub const NATIVE_BALANCE_2: u128 = 1_000_000_000_000u128;

    pub struct ContractInfo {
        pub contract_addr: String,
        pub contract_code_id: u64,
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(ADMIN),
                    vec![
                        Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(NATIVE_BALANCE),
                        },
                        Coin {
                            denom: NATIVE_DENOM_2.to_string(),
                            amount: Uint128::new(NATIVE_BALANCE_2),
                        },
                    ],
                )
                .unwrap();
        })
    }

    pub fn forging_gem_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(ForgingGemExecute, ForgingGemInstantiate, ForgingGemQuery);
        Box::new(contract)
    }

    pub fn dragon_collection_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw721_base::entry::execute,
            cw721_base::entry::instantiate,
            cw721_base::entry::query,
        );
        Box::new(contract)
    }

    pub fn auragon_collection_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw721_base::entry::execute,
            cw721_base::entry::instantiate,
            cw721_base::entry::query,
        );
        Box::new(contract)
    }

    pub fn shield_collection_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw721_base::entry::execute,
            cw721_base::entry::instantiate,
            cw721_base::entry::query,
        );
        Box::new(contract)
    }

    pub fn instantiate_contracts() -> (App, Vec<ContractInfo>) {
        // Create a new app instance
        let mut app = mock_app();
        // Create a vector to store all contract info ([halo factory - [0])
        let mut contract_info_vec: Vec<ContractInfo> = Vec::new();

        // store code of all contracts to the app and get the code ids
        let dragon_collection_code_id = app.store_code(dragon_collection_contract_template()); // [0]
        let auragon_collection_code_id = app.store_code(auragon_collection_contract_template()); // [1]
        let shield_collection_code_id = app.store_code(shield_collection_contract_template()); // [2]
        let forging_gem_code_id = app.store_code(forging_gem_contract_template()); // [3]

        // dragon collection contract
        // create instantiate message for contract
        let dragon_collection_instantiate_msg = Cw721InstantiateMsg {
            name: "Dragon Collection".to_string(),
            symbol: "DRAGON".to_string(),
            minter: ADMIN.to_string(),
        };

        // instantiate the contract
        let dragon_collection_contract_addr = app
            .instantiate_contract(
                dragon_collection_code_id,
                Addr::unchecked(ADMIN),
                &dragon_collection_instantiate_msg,
                &[],
                "test dragon collection",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: dragon_collection_contract_addr.to_string(),
            contract_code_id: dragon_collection_code_id,
        });

        // auragon collection contract
        // create instantiate message for contract
        let auragon_collection_instantiate_msg = Cw721InstantiateMsg {
            name: "Auragon Collection".to_string(),
            symbol: "AURAGON".to_string(),
            minter: ADMIN.to_string(),
        };

        // instantiate the contract
        let auragon_collection_contract_addr = app
            .instantiate_contract(
                auragon_collection_code_id,
                Addr::unchecked(ADMIN),
                &auragon_collection_instantiate_msg,
                &[],
                "test auragon collection",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: auragon_collection_contract_addr.to_string(),
            contract_code_id: auragon_collection_code_id,
        });

        // shield collection contract
        // create instantiate message for contract
        let shield_collection_instantiate_msg = Cw721InstantiateMsg {
            name: "Shield Collection".to_string(),
            symbol: "SHIELD".to_string(),
            minter: ADMIN.to_string(),
        };

        // instantiate the contract
        let shield_collection_contract_addr = app
            .instantiate_contract(
                shield_collection_code_id,
                Addr::unchecked(ADMIN),
                &shield_collection_instantiate_msg,
                &[],
                "test shield collection",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: shield_collection_contract_addr.to_string(),
            contract_code_id: shield_collection_code_id,
        });

        // forging gem contract
        // create instantiate message for contract
        let forging_gem_instantiate_msg = ForgingGemInstantiateMsg {
            random_seed: "46FAF1CD4845AB7C5A9DAA7D272259682BF84176A2658DE67CB1317A22134973"
                .to_string(),
            nois_proxy: "aura14s9rm7fneysy7srqcq8mrgk3us9alqwpes3mudshp03z2ktvltaqgux4xg"
                .to_string(),
            dragon_collection: dragon_collection_contract_addr.to_string(),
            auragon_collection: auragon_collection_contract_addr.to_string(),
            shield_collection: shield_collection_contract_addr.to_string(),
            white_gem_work_power: [
                Decimal::from_str("2").unwrap(),
                Decimal::from_str("3").unwrap(),
                Decimal::from_str("5").unwrap(),
                Decimal::from_str("8").unwrap(),
                Decimal::from_str("13").unwrap(),
                Decimal::from_str("21").unwrap(),
                Decimal::from_str("34").unwrap(),
            ],
            white_gem_uri: [
                "https://ipfs.io/ipfs/W1".to_string(),
                "https://ipfs.io/ipfs/W2".to_string(),
                "https://ipfs.io/ipfs/W3".to_string(),
                "https://ipfs.io/ipfs/W4".to_string(),
                "https://ipfs.io/ipfs/W5".to_string(),
                "https://ipfs.io/ipfs/W6".to_string(),
                "https://ipfs.io/ipfs/W7".to_string(),
            ],
            blue_gem_work_power: [
                Decimal::from_str("22.5").unwrap(),
                Decimal::from_str("33.75").unwrap(),
                Decimal::from_str("56.25").unwrap(),
                Decimal::from_str("90").unwrap(),
                Decimal::from_str("146.25").unwrap(),
                Decimal::from_str("236.25").unwrap(),
                Decimal::from_str("382.5").unwrap(),
            ],
            blue_gem_uri: [
                "https://ipfs.io/ipfs/B1".to_string(),
                "https://ipfs.io/ipfs/B2".to_string(),
                "https://ipfs.io/ipfs/B3".to_string(),
                "https://ipfs.io/ipfs/B4".to_string(),
                "https://ipfs.io/ipfs/B5".to_string(),
                "https://ipfs.io/ipfs/B6".to_string(),
                "https://ipfs.io/ipfs/B7".to_string(),
            ],
            gold_gem_work_power: [
                Decimal::from_str("2").unwrap(),
                Decimal::from_str("3").unwrap(),
                Decimal::from_str("5").unwrap(),
                Decimal::from_str("8").unwrap(),
                Decimal::from_str("13").unwrap(),
                Decimal::from_str("21").unwrap(),
                Decimal::from_str("34").unwrap(),
            ],
            gold_gem_uri: [
                "https://ipfs.io/ipfs/G1".to_string(),
                "https://ipfs.io/ipfs/G2".to_string(),
                "https://ipfs.io/ipfs/G3".to_string(),
                "https://ipfs.io/ipfs/G4".to_string(),
                "https://ipfs.io/ipfs/G5".to_string(),
                "https://ipfs.io/ipfs/G6".to_string(),
                "https://ipfs.io/ipfs/G7".to_string(),
            ],
            red_gem_work_power: [
                Decimal::from_str("22.5").unwrap(),
                Decimal::from_str("33.75").unwrap(),
                Decimal::from_str("56.25").unwrap(),
                Decimal::from_str("90").unwrap(),
                Decimal::from_str("146.25").unwrap(),
                Decimal::from_str("236.25").unwrap(),
                Decimal::from_str("382.5").unwrap(),
            ],
            red_gem_uri: [
                "https://ipfs.io/ipfs/R1".to_string(),
                "https://ipfs.io/ipfs/R2".to_string(),
                "https://ipfs.io/ipfs/R3".to_string(),
                "https://ipfs.io/ipfs/R4".to_string(),
                "https://ipfs.io/ipfs/R5".to_string(),
                "https://ipfs.io/ipfs/R6".to_string(),
                "https://ipfs.io/ipfs/R7".to_string(),
            ],
            shield_uri: "https://ipfs.io/ipfs/S1".to_string(),
            gem_ratio: [
                Decimal::from_str("0.9").unwrap(),
                Decimal::from_str("0.08").unwrap(),
                Decimal::from_str("0.015").unwrap(),
                Decimal::from_str("0.005").unwrap(),
            ],
            gem_work_load: [
                Decimal::from_str("0.1").unwrap(),
                Decimal::from_str("0.2").unwrap(),
                Decimal::from_str("0.3").unwrap(),
                Decimal::from_str("0.4").unwrap(),
                Decimal::from_str("0.5").unwrap(),
                Decimal::from_str("0.6").unwrap(),
            ],
        };

        // instantiate the contract
        let forging_gem_contract_addr = app
            .instantiate_contract(
                forging_gem_code_id,
                Addr::unchecked(ADMIN),
                &forging_gem_instantiate_msg,
                &[],
                "test forging gem",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: forging_gem_contract_addr.to_string(),
            contract_code_id: forging_gem_code_id,
        });

        (app, contract_info_vec)
    }


}