use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

// we need a wallet to control the contract
pub const CONTROLLER: Item<Addr> = Item::new("controller");

// a code_id of cw721 contract
pub const CW721_CODE_ID: Item<u64> = Item::new("cw721-code-id");

// the original collection mapping
pub const ORIGINAL_COLLECTIONS: Map<Addr, u64> = Map::new("original-collections");

// the mirrored collection mapping
pub const MIRRORED_COLLECTIONS: Map<Addr, u64> = Map::new("mirrored-collections");

// the wrap data mapping
pub const WRAP_DATA: Map<u64, WrapData> = Map::new("wrap-data");

// total wrap count
pub const TOTAL_WRAPPED: Item<u64> = Item::new("total-wrapped");

#[cw_serde]
pub struct WrapData {
    pub original_collection: Addr,
    pub mirrored_collection: Addr,
    pub mirrored_data: MirroredData,
    pub active: bool,
}

/// the information of mirrored data
/// this data will be used to override the original data
#[cw_serde]
pub struct MirroredData {
    pub collection_name: Option<String>,
    pub collection_symbol: Option<String>,
    pub description: Option<String>,
    pub token_data: Option<TokenData>,
}

#[cw_serde]
pub struct TokenData {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub base_uri: Option<String>,
    pub external_url: Option<String>,
}
