#[cfg(test)]
pub mod env {
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw20::MinterResponse;
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::contract::{
        execute as ForgingGemExecute, instantiate as ForgingGemInstantiate, query as ForgingGemQuery,
    };

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


}