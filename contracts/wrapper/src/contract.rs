#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, QueryRequest,
    Reply, ReplyOn, Response, StdResult, SubMsg, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw721::{
    ContractInfoResponse as Cw721ContractInfoResponse, Cw721ExecuteMsg, Cw721QueryMsg,
    NftInfoResponse as Cw721NftInfoResponse, OwnerOfResponse,
};
use cw721_base::msg::{ExecuteMsg as Cw721BaseExecuteMsg, InstantiateMsg as Cw721InstantiateMsg};
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    Extension, MirroredData, WrapData, CONTROLLER, CW721_CODE_ID, MIRRORED_COLLECTIONS,
    ORIGINAL_COLLECTIONS, TOTAL_WRAPPED, WRAP_DATA,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:wrapper";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // update controller
    CONTROLLER.save(deps.storage, &deps.api.addr_validate(&msg.controller)?)?;

    // update cw721 code id
    CW721_CODE_ID.save(deps.storage, &msg.cw721_code_id)?;

    // init total wrapped
    TOTAL_WRAPPED.save(deps.storage, &0u64)?;

    // now we instantiate the cw20 contract
    Ok(Response::new().add_attributes([
        ("method", "instantiate"),
        ("controller", &msg.controller),
        ("cw721_code_id", &msg.cw721_code_id.to_string()),
    ]))
}

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Wrap {
            collection_address,
            token_ids,
        } => execute_wrap(deps, env, info, collection_address, token_ids),
        ExecuteMsg::Unwrap {
            collection_address,
            token_ids,
        } => execute_unwrap(deps, env, info, collection_address, token_ids),
        ExecuteMsg::RegisterCollection {
            original_collection,
            new_collection,
        } => execute_register_collection(deps, env, info, original_collection, new_collection),
    }
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Controller {} => to_json_binary(&query_controller(deps)?),
        QueryMsg::OriginalCollectionInfo { collection_address } => to_json_binary(
            &ORIGINAL_COLLECTIONS
                .load(deps.storage, deps.api.addr_validate(&collection_address)?)?,
        ),
        QueryMsg::MirroredCollectionInfo { collection_address } => to_json_binary(
            &MIRRORED_COLLECTIONS
                .load(deps.storage, deps.api.addr_validate(&collection_address)?)?,
        ),
        QueryMsg::WrapData { index } => to_json_binary(&WRAP_DATA.load(deps.storage, index)?),
    }
}

/// Handling submessage reply.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let wrap_data_index = msg.id;

    let reply_msg = parse_reply_instantiate_data(msg).unwrap();

    // save the rest of the data to wrap data
    let mut wrap_data = WRAP_DATA.load(deps.storage, wrap_data_index)?;
    wrap_data.mirrored_collection = deps.api.addr_validate(&reply_msg.contract_address)?;
    wrap_data.active = true;
    WRAP_DATA.save(deps.storage, wrap_data_index, &wrap_data)?;

    // update mirrored collection mapping
    MIRRORED_COLLECTIONS.save(
        deps.storage,
        deps.api.addr_validate(&reply_msg.contract_address)?,
        &wrap_data_index,
    )?;

    Ok(Response::new().add_attributes([
        ("method", "reply"),
        ("wrap_data", &wrap_data_index.to_string()),
        (
            "original_collection",
            wrap_data.original_collection.as_ref(),
        ),
        ("mirrored_collection", &reply_msg.contract_address),
    ]))
}

pub fn execute_register_collection(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    original_collection: String,
    new_collection: MirroredData,
) -> Result<Response, ContractError> {
    // only controller can register collection
    if info.sender != CONTROLLER.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }

    let mut res = Response::new();
    // if the original collection is not in the list, then we must create new mirror for it
    if !ORIGINAL_COLLECTIONS.has(deps.storage, deps.api.addr_validate(&original_collection)?) {
        // query contract info of original collection
        let contract_info_response: StdResult<Cw721ContractInfoResponse> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: original_collection.clone(),
                msg: to_json_binary(&Cw721QueryMsg::ContractInfo {})?,
            }));

        match contract_info_response {
            Ok(contract_info) => {
                let mirrored_name = new_collection
                    .collection_name
                    .clone()
                    .unwrap_or(contract_info.name);
                let mirrored_symbol = new_collection
                    .collection_symbol
                    .clone()
                    .unwrap_or(contract_info.symbol);

                // cw721 instantiate msg
                let cw721_instantiation_msg = Cw721InstantiateMsg {
                    name: mirrored_name,
                    symbol: mirrored_symbol,
                    minter: env.contract.address.to_string(),
                };

                // increase total wrapped
                let mut total_wrapped = TOTAL_WRAPPED.load(deps.storage)?;
                total_wrapped += 1;
                TOTAL_WRAPPED.save(deps.storage, &total_wrapped)?;

                // instantiate new mirror collection
                res = res.add_submessage(SubMsg {
                    id: total_wrapped,
                    gas_limit: None,
                    msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                        admin: Some(CONTROLLER.load(deps.storage).unwrap().to_string()),
                        code_id: CW721_CODE_ID.load(deps.storage)?,
                        msg: to_json_binary(&cw721_instantiation_msg)?,
                        funds: vec![],
                        label: format!("Intantiate mirror collection for {}", original_collection),
                    }),
                    reply_on: ReplyOn::Success,
                });

                // now update all data
                // update original collection mapping
                ORIGINAL_COLLECTIONS.save(
                    deps.storage,
                    deps.api.addr_validate(&original_collection)?,
                    &total_wrapped,
                )?;
                // the mirrored collection mapping will be updated in reply
                // update wrap data with the status of active = false
                let wrap_data = WrapData {
                    original_collection: deps.api.addr_validate(&original_collection)?,
                    mirrored_collection: Addr::unchecked(""),
                    mirrored_data: MirroredData {
                        collection_name: new_collection.collection_name,
                        collection_symbol: new_collection.collection_symbol,
                        base_uri: new_collection.base_uri,
                    },
                    active: false,
                };
                WRAP_DATA.save(deps.storage, total_wrapped, &wrap_data)?;
            }
            Err(_) => {
                return Err(ContractError::Unauthorized {});
            }
        }
    }

    Ok(res.add_attributes([
        ("method", "register_collection"),
        ("original_collection", &original_collection),
    ]))
}

pub fn execute_wrap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_address: String,
    token_ids: Vec<String>,
) -> Result<Response, ContractError> {
    // if the collection is not registered, then return error
    if !ORIGINAL_COLLECTIONS.has(deps.storage, deps.api.addr_validate(&collection_address)?) {
        return Err(ContractError::CollectionNotAllowed {});
    }

    // if the original collection status is not active, then return error
    let wrap_data = WRAP_DATA.load(
        deps.storage,
        ORIGINAL_COLLECTIONS.load(deps.storage, deps.api.addr_validate(&collection_address)?)?,
    )?;
    if !wrap_data.active {
        return Err(ContractError::CollectionDeactivated {});
    }

    let mut res = Response::new();

    for token_id in token_ids.iter() {
        // if the token id are not owned by the sender, then return error
        let owner_response: StdResult<OwnerOfResponse> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: collection_address.clone(),
                msg: to_json_binary(&Cw721QueryMsg::OwnerOf {
                    token_id: token_id.to_string(),
                    include_expired: None,
                })?,
            }));
        match owner_response {
            Ok(owner) => {
                if owner.owner != info.sender {
                    return Err(ContractError::NotOwnedBySender {
                        val: token_id.to_string(),
                    });
                }
            }
            Err(_) => {
                return Err(ContractError::NotOwnedBySender {
                    val: token_id.to_string(),
                });
            }
        }

        // transfer the token to the contract
        res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: collection_address.clone(),
            msg: to_json_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: env.contract.address.to_string(),
                token_id: token_id.to_string(),
            })?,
            funds: vec![],
        }));

        // query info of the token
        let token_info_response: StdResult<Cw721NftInfoResponse<Extension>> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: collection_address.clone(),
                msg: to_json_binary(&Cw721QueryMsg::NftInfo {
                    token_id: token_id.to_string(),
                })?,
            }));

        // decide the token uri and extension
        let (token_uri, _extension) = match token_info_response {
            Ok(token_info) => {
                // if the token_uri of mirrored data is not empty, then use it
                if wrap_data.mirrored_data.base_uri.is_some() {
                    (
                        Some(format!(
                            "{}{}.json",
                            wrap_data.mirrored_data.base_uri.clone().unwrap(),
                            token_id
                        )),
                        token_info.extension,
                    )
                } else {
                    (token_info.token_uri, token_info.extension)
                }
            }
            Err(_) => {
                return Err(ContractError::CustomError {
                    val: format!("Cannot query token info of {}", token_id),
                });
            }
        };

        // mint mirrored token to the sender
        let mint_msg: Cw721BaseExecuteMsg<_, Extension> = Cw721BaseExecuteMsg::Mint {
            owner: info.sender.to_string(),
            token_id: token_id.to_string(),
            token_uri,
            extension: Empty::default(),
        };
        res = res.add_message(WasmMsg::Execute {
            contract_addr: wrap_data.mirrored_collection.to_string(),
            msg: to_json_binary(&mint_msg)?,
            funds: vec![],
        });
    }

    Ok(res.add_attributes([
        ("method", "wrap"),
        ("collection_address", &collection_address),
        ("token_ids", &token_ids.join(",")),
    ]))
}

pub fn execute_unwrap(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_address: String,
    token_ids: Vec<String>,
) -> Result<Response, ContractError> {
    // if the collection is not registered, then return error
    if !MIRRORED_COLLECTIONS.has(deps.storage, deps.api.addr_validate(&collection_address)?) {
        return Err(ContractError::CollectionNotAllowed {});
    }

    // if the mirrored collection status is not active, then return error
    let wrap_data = WRAP_DATA.load(
        deps.storage,
        MIRRORED_COLLECTIONS.load(deps.storage, deps.api.addr_validate(&collection_address)?)?,
    )?;
    if !wrap_data.active {
        return Err(ContractError::CollectionDeactivated {});
    }

    let mut res = Response::new();

    for token_id in token_ids.iter() {
        // if the token id are not owned by the sender, then return error
        let owner_response: StdResult<OwnerOfResponse> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: collection_address.clone(),
                msg: to_json_binary(&Cw721QueryMsg::OwnerOf {
                    token_id: token_id.to_string(),
                    include_expired: None,
                })?,
            }));
        match owner_response {
            Ok(owner) => {
                if owner.owner != info.sender {
                    return Err(ContractError::NotOwnedBySender {
                        val: token_id.to_string(),
                    });
                }
            }
            Err(_) => {
                return Err(ContractError::NotOwnedBySender {
                    val: token_id.to_string(),
                });
            }
        }

        // burn the mirrored token
        res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: collection_address.clone(),
            msg: to_json_binary(&Cw721ExecuteMsg::Burn {
                token_id: token_id.to_string(),
            })?,
            funds: vec![],
        }));

        // transfer the original token to the sender
        res = res.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: wrap_data.original_collection.to_string(),
            msg: to_json_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: info.sender.to_string(),
                token_id: token_id.to_string(),
            })?,
            funds: vec![],
        }));
    }

    Ok(res.add_attributes([
        ("method", "unwrap"),
        ("collection_address", &collection_address),
        ("token_ids", &token_ids.join(",")),
    ]))
}

pub fn query_controller(deps: Deps) -> StdResult<Addr> {
    CONTROLLER.load(deps.storage)
}
