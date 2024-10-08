// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use coerce::actor::message::Message;
use serde::{Deserialize, Serialize};

use moveos_types::h256::H256;

/// The batch in Rooch is constructed by the batch submitter, representing a batch of transactions, mapping to a L2 block
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Batch {
    /// each batch maps to a L2 block
    pub block_number: u128,
    /// How many transactions in the batch
    pub tx_count: u64,
    /// The previous tx accumulator root of the block
    pub prev_tx_accumulator_root: H256,
    /// The tx accumulator root after the last transaction append to the accumulator
    pub tx_accumulator_root: H256,

    /// sha256h of data
    pub batch_hash: H256,
    /// encoded tx(LedgerTransaction) list
    pub data: Vec<u8>,
}

impl Message for Batch {
    type Result = Result<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PutBatchInternalDAMessage {
    pub batch: Batch,
    // TODO add put policy
}

impl Message for PutBatchInternalDAMessage {
    type Result = Result<()>;
}
