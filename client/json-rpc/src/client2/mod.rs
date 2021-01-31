// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::{Error, Result};
use serde::{Deserialize, Serialize};

mod blocking;
mod client;
mod request;
mod response;

pub use blocking::BlockingClient;
pub use client::Client;
pub use request::{JsonRpcRequest, MethodRequest};
pub use response::{MethodResponse, Response, State};

#[derive(Debug, Deserialize, Serialize)]
enum JsonRpcVersion {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    Submit,
    GetMetadata,
    GetAccount,
    GetTransactions,
    GetAccountTransaction,
    GetAccountTransactions,
    GetEvents,
    GetCurrencies,
    GetNetworkStatus,

    //
    // Experimental APIs
    //
    GetStateProof,
    GetAccountStateWithProof,
    GetTransactionsWithProofs,
    GetEventsWithProofs,
}

fn validate(
    resp: &diem_json_rpc_types::response::JsonRpcResponse,
) -> Result<(u64, State, serde_json::Value)> {
    if resp.jsonrpc != "2.0" {
        return Err(Error::InvalidRpcResponse(resp.clone()));
    }
    let id = get_id(resp)?;

    if let Some(err) = &resp.error {
        return Err(Error::JsonRpcError(err.clone()));
    }

    // Result being empty is an acceptable response
    let result = resp.result.clone().unwrap_or(serde_json::Value::Null);

    let state = State::from_response(resp);
    Ok((id, state, result))
}

fn validate_batch(
    requests: &[JsonRpcRequest],
    raw_responses: Vec<diem_json_rpc_types::response::JsonRpcResponse>,
) -> Result<Vec<Result<Response<MethodResponse>>>> {
    let mut responses = HashMap::new();
    for raw_response in raw_responses {
        let id = get_id(&raw_response)?;
        let response = validate(&raw_response);

        responses.insert(id, response);
    }

    let mut result = Vec::new();

    for request in requests {
        let response = if let Some(response) = responses.remove(&request.id()) {
            response
        } else {
            return Err(Error::unexpected_invalid_batch_response(vec![]));
        };

        let response = response.and_then(|(_id, state, result)| {
            MethodResponse::from_json(request.method(), result)
                .map(|result| Response::new(result, state))
        });

        result.push(response);
    }

    if !responses.is_empty() {
        return Err(Error::unexpected_invalid_batch_response(vec![]));
    }

    Ok(result)
}

fn get_id(resp: &diem_json_rpc_types::response::JsonRpcResponse) -> Result<u64> {
    let id = if let Some(id) = &resp.id {
        if let Ok(index) = serde_json::from_value::<u64>(id.clone()) {
            index
        } else {
            return Err(Error::unexpected_invalid_response_id_type(resp.clone()));
        }
    } else {
        return Err(Error::unexpected_response_id_not_found(resp.clone()));
    };

    Ok(id)
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum BatchResponse {
    Success(Vec<diem_json_rpc_types::response::JsonRpcResponse>),
    Error(Box<diem_json_rpc_types::response::JsonRpcResponse>),
}

impl BatchResponse {
    pub fn success(self) -> Result<Vec<diem_json_rpc_types::response::JsonRpcResponse>> {
        match self {
            BatchResponse::Success(inner) => Ok(inner),
            BatchResponse::Error(e) => Err(Error::JsonRpcError(e.error.unwrap())),
        }
    }
}
