// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use jsonrpsee::RpcModule;
use async_trait::async_trait;
use tracing::{info, instrument};
use crate::api::RoochRpcModule;
use crate::jsonrpc_types::coin::Balance;
use crate::response::JsonResponse;

#[rpc(server, client, namespace = "rooch")]
pub trait AccountApi {
    #[method(name = "account.get_balance")]
    async fn get_balance(&self, token: String) -> RpcResult<JsonResponse<Balance>>;
}

pub struct AccountServer {
}

impl AccountServer {
    pub fn new() -> Self {
        Self {}
    }

    #[instrument(skip(self))]
    async fn get_balance(&self, token: String) -> RpcResult<JsonResponse<Balance>> {
        info!("get_balance");

        Ok(JsonResponse::ok(Balance {
            // coin_type: "Rooch".to_string(),
            coin_type: token,
            total_balance: 1005,
        }))
    }
}

#[async_trait]
impl AccountApiServer for AccountServer {
    async fn get_balance(&self, token: String) -> RpcResult<JsonResponse<Balance>> {
        Ok(self.get_balance(token).await?)
    }
}

impl RoochRpcModule for AccountServer {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
