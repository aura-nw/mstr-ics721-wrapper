use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::MirroredData;

/// Message type for `instantiate` entry_point
/// Maybe we don't need a new cw20 contract, just use the cw20-base contract
#[cw_serde]
pub struct InstantiateMsg {
    pub controller: String,
    pub cw721_code_id: u64,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    Wrap {
        collection_address: String,
        token_ids: Vec<String>,
    },
    Unwrap {
        token_ids: Vec<String>,
    },
    RegisterCollection {
        original_collection: String,
        new_collection: MirroredData,
    },
}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
    Controller {},
}

#[cw_serde]
pub struct ReceiverResponse {
    pub name: String,
    pub address: String,
}

#[cw_serde]
pub struct ExchangingInfoResponse {
    pub accepted_denom: String,
    pub token_address: String,
    pub price_feed: String,
}
