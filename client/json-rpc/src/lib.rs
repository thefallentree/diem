// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod blocking;
mod client;
mod error;
mod response;

pub use blocking::JsonRpcClient;
pub use client::{
    get_response_from_batch, process_batch_response, JsonRpcAsyncClient, JsonRpcAsyncClientError,
    JsonRpcBatch,
};
pub use diem_json_rpc_types::{errors, views};
pub use diem_types::{account_address::AccountAddress, transaction::SignedTransaction};
pub use error::{Error, Result, UnexpectedError, WaitForTransactionError};
pub use response::{JsonRpcResponse, ResponseAsView};

// new implementation module

pub mod async_client;

// Unified work

pub mod client2;

pub use client2::{BlockingClient, Client, MethodRequest, MethodResponse};
