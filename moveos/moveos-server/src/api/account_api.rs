// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use jsonrpsee::RpcModule;

use crate::api::account::AccountAPIServer;
use crate::api::RoochRpcModule;
use tracing::{info, instrument};

use crate::jsonrpc_types::coin::Balance;

pub struct AccountAPI {
}
impl AccountAPI {
    pub fn new() -> Self {
        Self {}
    }

    #[instrument(skip(self))]
    async fn get_balance(&self, token: String) -> RpcResult<Balance> {
        info!("get_balance");
        Ok(Balance {
            // coin_type: "Rooch".to_string(),
            coin_type: token,
            total_balance: 1005,
        })
    }
}

#[async_trait]
impl AccountAPIServer for AccountAPI {
    async fn get_balance(&self, token: String) -> RpcResult<Balance> {
        Ok(self.get_balance(token).await?)
    }
}

impl RoochRpcModule for AccountAPI {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }
}
