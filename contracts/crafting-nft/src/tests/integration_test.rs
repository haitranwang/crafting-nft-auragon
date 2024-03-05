#![cfg(test)]
mod tests {
    const INIT_1000_000_NATIVE_BALANCE_2: u128 = 1_000_000_000_000u128;
    mod execute_proper_operation {
        use crate::state::{Config, GemInfo, GemMetadata, CONFIG, AURAGON_LATEST_TOKEN_ID, RANDOM_SEED};
        use crate::tests::integration_test::tests::INIT_1000_000_NATIVE_BALANCE_2;
        use cosmwasm_std::{
            from_json, to_json_binary, Addr, BalanceResponse as BankBalanceResponse, BankQuery,
            BlockInfo, Coin, Decimal, Querier, QueryRequest, Uint128, WasmQuery,
        };
        use cw_multi_test::Executor;
        use crate::msg::{ExecuteMsg as ForgingGemExecuteMsg, InstantiateMsg as ForgingGemInstantiateMsg, QueryMsg as ForgingGemQueryMsg};
        use crate::tests::env_setup::env::{forging_gem_contract_template, instantiate_contracts, ADMIN, NATIVE_DENOM_2};

        #[test]
        fn proper_operation() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            // get dragon_collection contract address
            let dragon_collection_addr = &contracts[0].contract_addr;
            // get auragon_collection contract address
            let auragon_collection_addr = &contracts[1].contract_addr;
            // get shield_collection contract address
            let shield_collection_addr = &contracts[2].contract_addr;
            // get forging_gem contract address
            let forging_gem_addr = &contracts[3].contract_addr;

            // query balance of ADMIN in native token
            let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance {
                address: ADMIN.to_string(),
                denom: NATIVE_DENOM_2.to_string(),
            });
            let res = app.raw_query(&to_json_binary(&req).unwrap()).unwrap().unwrap();
            let balance: BankBalanceResponse = from_json(&res).unwrap();

            // It should be 1_000_000 NATIVE_DENOM_2 as minting happened
            assert_eq!(
                balance.amount.amount,
                Uint128::from(INIT_1000_000_NATIVE_BALANCE_2)
            );

        }
    }
}