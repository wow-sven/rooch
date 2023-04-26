// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use crate::jsonrpc_types::coin::Balance;

#[rpc(server, client, namespace = "rooch")]
pub trait AccountAPI {
    #[method(name = "account.get_balance")]
    async fn get_balance(&self, token: String) -> RpcResult<Balance>;
}
