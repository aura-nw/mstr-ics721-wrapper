// #[cfg(test)]
// pub mod env {
//     use cosmwasm_std::{Addr, Coin, Empty, Uint128};

//     use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

//     use crate::contract::{
//         execute as MinterExecute, instantiate as MinterInstantiate, query as MinterQuery,
//         reply as MinterReply,
//     };

//     use price_feed::contract::{
//         execute as PriceFeedExecute, instantiate as PriceFeedInstantiate, query as PriceFeedQuery,
//     };

//     use cw20_base::contract::{
//         execute as Cw20Execute, instantiate as Cw20Instantiate, query as Cw20Query,
//     };

//     use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;

//     use crate::msg::InstantiateMsg as MinterInstantiateMsg;
//     use price_feed::msg::InstantiateMsg as PriceFeedInstantiateMsg;

//     pub const ADMIN: &str = "aura1000000000000000000000000000000000admin";
//     pub const USER1: &str = "aura1000000000000000000000000000000000user1";
//     pub const AURA: &str = "aura10000000000000000000000000000000000aura";
//     pub const CONTROLLER: &str = "aura10000000000000000000000000000controller";
//     pub const CONTROLLER_FAKE: &str = "aura10000000000000000000000000000fake";

//     pub const NATIVE_DENOM: &str = "uaura";
//     pub const NATIVE_BALANCE: u128 = 1_000_000_000_000u128;

//     pub const NATIVE_DENOM_2: &str = "utaura";
//     pub const NATIVE_BALANCE_2: u128 = 1_000_000_000_000u128;

//     pub struct ContractInfo {
//         pub contract_addr: String,
//         pub contract_code_id: u64,
//     }

//     // create app instance and init balance of NATIVE token for admin
//     fn mock_app() -> App {
//         AppBuilder::new().build(|router, _, storage| {
//             router
//                 .bank
//                 .init_balance(
//                     storage,
//                     &Addr::unchecked(ADMIN),
//                     vec![
//                         Coin {
//                             denom: NATIVE_DENOM.to_string(),
//                             amount: Uint128::new(NATIVE_BALANCE),
//                         },
//                         Coin {
//                             denom: NATIVE_DENOM_2.to_string(),
//                             amount: Uint128::new(NATIVE_BALANCE_2),
//                         },
//                     ],
//                 )
//                 .unwrap();
//         })
//     }

//     // create minter contract
//     pub fn minter_contract_template() -> Box<dyn Contract<Empty>> {
//         let contract = ContractWrapper::new(MinterExecute, MinterInstantiate, MinterQuery)
//             .with_reply(MinterReply);
//         Box::new(contract)
//     }

//     // create price feed contract
//     pub fn price_feed_contract_template() -> Box<dyn Contract<Empty>> {
//         let contract = ContractWrapper::new(PriceFeedExecute, PriceFeedInstantiate, PriceFeedQuery);
//         Box::new(contract)
//     }

//     // create cw20 contract
//     pub fn cw20_contract_template() -> Box<dyn Contract<Empty>> {
//         let contract = ContractWrapper::new(Cw20Execute, Cw20Instantiate, Cw20Query);
//         Box::new(contract)
//     }

//     pub fn instantiate_contracts() -> (App, Vec<ContractInfo>) {
//         // Create a new app instance
//         let mut app = mock_app();
//         // Create a vector to store all contract info ([halo factory - [0])
//         let mut contract_info_vec: Vec<ContractInfo> = Vec::new();

//         // store code of all contracts to the app and get the code ids
//         let minter_contract_code_id = app.store_code(minter_contract_template());
//         let price_feed_contract_code_id = app.store_code(price_feed_contract_template());
//         let cw20_contract_code_id = app.store_code(cw20_contract_template());

//         // instantiate price feed contract
//         let price_feed_contract_addr = app
//             .instantiate_contract(
//                 price_feed_contract_code_id,
//                 Addr::unchecked(ADMIN),
//                 &PriceFeedInstantiateMsg {
//                     controller: CONTROLLER.to_string(),
//                     decimals: 6,
//                     description: "AURA / VND".to_string(),
//                 },
//                 &[],
//                 "test instantiate contract",
//                 None,
//             )
//             .unwrap();
//         // add contract info to the vector
//         contract_info_vec.push(ContractInfo {
//             contract_addr: price_feed_contract_addr.to_string(),
//             contract_code_id: price_feed_contract_code_id,
//         });

//         // instantiate minter contract
//         let minter_contract_addr = app
//             .instantiate_contract(
//                 minter_contract_code_id,
//                 Addr::unchecked(ADMIN),
//                 &MinterInstantiateMsg {
//                     receiver_name: "aura".to_string(),
//                     receiver_address: AURA.to_string(),
//                     accepted_denom: NATIVE_DENOM.to_string(),
//                     price_feed: price_feed_contract_addr.to_string(),
//                     token_code_id: cw20_contract_code_id,
//                     token_instantiation_msg: Cw20InstantiateMsg {
//                         name: "Stable Token".to_string(),
//                         symbol: "STV".to_string(),
//                         decimals: 6,
//                         initial_balances: vec![],
//                         mint: None,
//                         marketing: None,
//                     },
//                 },
//                 &[],
//                 "test instantiate contract",
//                 None,
//             )
//             .unwrap();

//         // add contract info to the vector
//         contract_info_vec.push(ContractInfo {
//             contract_addr: minter_contract_addr.to_string(),
//             contract_code_id: minter_contract_code_id,
//         });

//         (app, contract_info_vec)
//     }

//     #[test]
//     fn test_instantiate_contracts() {
//         let (_app, contract_info_vec) = instantiate_contracts();

//         // check if all contracts are instantiated
//         assert_eq!(contract_info_vec.len(), 2);
//     }
// }
