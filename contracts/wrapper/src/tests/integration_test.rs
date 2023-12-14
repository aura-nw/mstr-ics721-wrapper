// #![cfg(test)]
// mod tests {
//     use crate::msg::ExecuteMsg;
//     use crate::tests::env_setup::env::{
//         instantiate_contracts, ADMIN, AURA, CONTROLLER, CONTROLLER_FAKE, NATIVE_DENOM, USER1,
//     };
//     use cosmwasm_std::{coins, Addr, Uint128};
//     use cw_multi_test::Executor;
//     use price_feed::msg::{
//         ExecuteMsg as PriceFeedExecuteMsg, QueryMsg as PriceFeedQueryMsg, RoundDataResponse,
//     };

//     mod price_feed_testing {
//         use super::*;

//         #[test]
//         fn only_owner_can_change_controller() {
//             // get integration test app and contracts
//             let (mut app, contracts) = instantiate_contracts();
//             let price_feed_contract_addr = &contracts[0].contract_addr;

//             // prepare the update controller message
//             let update_controller_msg = PriceFeedExecuteMsg::UpdateController {
//                 controller: CONTROLLER_FAKE.to_string(),
//             };

//             let res = app.execute_contract(
//                 Addr::unchecked(USER1),
//                 Addr::unchecked(price_feed_contract_addr),
//                 &update_controller_msg,
//                 &[],
//             );
//             assert!(res.is_err());

//             let res = app.execute_contract(
//                 Addr::unchecked(ADMIN),
//                 Addr::unchecked(price_feed_contract_addr),
//                 &update_controller_msg,
//                 &[],
//             );
//             assert!(res.is_ok());
//         }

//         #[test]
//         fn only_controller_can_update_answer() {
//             // get integration test app and contracts
//             let (mut app, contracts) = instantiate_contracts();
//             let price_feed_contract_addr = &contracts[0].contract_addr;

//             // prepare the update controller message
//             let update_answer_msg = PriceFeedExecuteMsg::UpdateRoundData { answer: 100000u64 };

//             let res = app.execute_contract(
//                 Addr::unchecked(USER1),
//                 Addr::unchecked(price_feed_contract_addr),
//                 &update_answer_msg,
//                 &[],
//             );
//             assert!(res.is_err());

//             let res = app.execute_contract(
//                 Addr::unchecked(ADMIN),
//                 Addr::unchecked(price_feed_contract_addr),
//                 &update_answer_msg,
//                 &[],
//             );
//             assert!(res.is_err());

//             let res = app.execute_contract(
//                 Addr::unchecked(CONTROLLER),
//                 Addr::unchecked(price_feed_contract_addr),
//                 &update_answer_msg,
//                 &[],
//             );
//             assert!(res.is_ok());

//             // query the price feed contract to check if the answer is updated
//             let query_msg = PriceFeedQueryMsg::LastestRoundData {};

//             let res: RoundDataResponse = app
//                 .wrap()
//                 .query_wasm_smart(price_feed_contract_addr, &query_msg)
//                 .unwrap();
//             assert_eq!(res.answer, Uint128::from(100000u64));
//         }
//     }

//     mod minter {
//         use crate::msg::{ExchangingInfoResponse, QueryMsg};

//         use super::*;

//         #[test]
//         fn user_can_exchange_token() {
//             // get integration test app and contracts
//             let (mut app, contracts) = instantiate_contracts();
//             let minter_contract_addr = &contracts[1].contract_addr;
//             let price_feed_contract_addr = &contracts[0].contract_addr;

//             // prepare the update controller message
//             let update_answer_msg = PriceFeedExecuteMsg::UpdateRoundData {
//                 answer: 10000000u64,
//             };

//             let res = app.execute_contract(
//                 Addr::unchecked(CONTROLLER),
//                 Addr::unchecked(price_feed_contract_addr),
//                 &update_answer_msg,
//                 &[],
//             );
//             assert!(res.is_ok());

//             // ADMIN send 50 native token to USER1
//             let res = app.send_tokens(
//                 Addr::unchecked(ADMIN),
//                 Addr::unchecked(USER1),
//                 &coins(50, NATIVE_DENOM),
//             );
//             assert!(res.is_ok());

//             // prepare the exchange message
//             let exchange_msg = ExecuteMsg::Exchange {
//                 amount: Uint128::from(50u64),
//                 expected_received: Uint128::from(500000000u64),
//             };

//             let res = app.execute_contract(
//                 Addr::unchecked(USER1),
//                 Addr::unchecked(minter_contract_addr),
//                 &exchange_msg,
//                 &coins(50, NATIVE_DENOM),
//             );
//             assert!(res.is_ok());

//             // query information of token_address of minter contract
//             let query_msg = QueryMsg::ExchangingInfo {};
//             let exchanging_info_res: ExchangingInfoResponse = app
//                 .wrap()
//                 .query_wasm_smart(minter_contract_addr, &query_msg)
//                 .unwrap();

//             // query balance of USER1 in the token_address
//             let query_msg = cw20_base::msg::QueryMsg::Balance {
//                 address: USER1.to_string(),
//             };
//             let res: cw20::BalanceResponse = app
//                 .wrap()
//                 .query_wasm_smart(exchanging_info_res.token_address.clone(), &query_msg)
//                 .unwrap();

//             // the balance should be 0
//             assert_eq!(res.balance, Uint128::zero());

//             // query balance of AURA in the token_address
//             let query_msg = cw20_base::msg::QueryMsg::Balance {
//                 address: AURA.to_string(),
//             };
//             let res: cw20::BalanceResponse = app
//                 .wrap()
//                 .query_wasm_smart(exchanging_info_res.token_address, &query_msg)
//                 .unwrap();

//             // the balance should be 500000000
//             assert_eq!(res.balance, Uint128::from(500000000u64));
//         }
//     }
// }
