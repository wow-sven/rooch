// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::accumulator_store::{AccumulatorStore, TransactionAccumulatorStore};
use crate::meta_store::{MetaDBStore, MetaStore};
use crate::state_store::{StateDBStore, StateStore};
use crate::transaction_store::{TransactionDBStore, TransactionStore};
use accumulator::AccumulatorTreeStore;
use anyhow::Result;
use moveos_config::store_config::RocksdbConfig;
use moveos_config::DataDirPath;
use moveos_types::h256::H256;
use moveos_types::state::StateChangeSetExt;
use once_cell::sync::Lazy;
use prometheus::Registry;
use raw_store::metrics::DBMetrics;
use raw_store::rocks::RocksDB;
use raw_store::{ColumnFamilyName, StoreInstance};
use rooch_types::sequencer::SequencerInfo;
use rooch_types::transaction::LedgerTransaction;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::sync::Arc;

pub mod accumulator_store;
pub mod meta_store;
pub mod state_store;
#[cfg(test)]
mod tests;
pub mod transaction_store;

// pub const DEFAULT_COLUMN_FAMILY_NAME: ColumnFamilyName = "default";
pub const TRANSACTION_COLUMN_FAMILY_NAME: ColumnFamilyName = "transaction";
pub const TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME: ColumnFamilyName =
    "tx_sequence_info_mapping";
pub const META_SEQUENCER_INFO_COLUMN_FAMILY_NAME: ColumnFamilyName = "meta_sequencer_info";
pub const TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME: ColumnFamilyName = "transaction_acc_node";

pub const STATE_CHANGE_SET_COLUMN_FAMILY_NAME: ColumnFamilyName = "state_change_set";

///db store use cf_name vec to init
/// Please note that adding a column family needs to be added in vec simultaneously, remember！！
static VEC_COLUMN_FAMILY_NAME: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        TRANSACTION_COLUMN_FAMILY_NAME,
        TX_SEQUENCE_INFO_MAPPING_COLUMN_FAMILY_NAME,
        META_SEQUENCER_INFO_COLUMN_FAMILY_NAME,
        TX_ACCUMULATOR_NODE_COLUMN_FAMILY_NAME,
        STATE_CHANGE_SET_COLUMN_FAMILY_NAME,
    ]
});

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StoreMeta {}

impl StoreMeta {
    pub fn get_column_family_names() -> &'static [ColumnFamilyName] {
        &VEC_COLUMN_FAMILY_NAME
    }
}

#[derive(Clone)]
pub struct RoochStore {
    pub transaction_store: TransactionDBStore,
    pub meta_store: MetaDBStore,
    pub transaction_accumulator_store: AccumulatorStore<TransactionAccumulatorStore>,
    pub state_store: StateDBStore,
}

impl RoochStore {
    pub fn new(db_path: &Path, registry: &Registry) -> Result<Self> {
        let db_metrics = DBMetrics::get_or_init(registry).clone();
        let instance = StoreInstance::new_db_instance(
            RocksDB::new(
                db_path,
                StoreMeta::get_column_family_names().to_vec(),
                RocksdbConfig::default(),
            )?,
            db_metrics,
        );
        Self::new_with_instance(instance, registry)
    }

    pub fn new_with_instance(instance: StoreInstance, _registry: &Registry) -> Result<Self> {
        let store = Self {
            transaction_store: TransactionDBStore::new(instance.clone()),
            meta_store: MetaDBStore::new(instance.clone()),
            transaction_accumulator_store: AccumulatorStore::new_transaction_accumulator_store(
                instance.clone(),
            ),
            state_store: StateDBStore::new(instance),
        };
        Ok(store)
    }

    pub fn mock_rooch_store() -> Result<(Self, DataDirPath)> {
        let tmpdir = moveos_config::temp_dir();
        let registry = prometheus::Registry::new();

        //The testcases should hold the tmpdir to prevent the tmpdir from being deleted.
        Ok((Self::new(tmpdir.path(), &registry)?, tmpdir))
    }

    pub fn get_transaction_store(&self) -> &TransactionDBStore {
        &self.transaction_store
    }

    pub fn get_meta_store(&self) -> &MetaDBStore {
        &self.meta_store
    }

    pub fn get_transaction_accumulator_store(&self) -> Arc<dyn AccumulatorTreeStore> {
        Arc::new(self.transaction_accumulator_store.clone())
    }

    pub fn get_state_store(&self) -> &StateDBStore {
        &self.state_store
    }
}

impl Display for RoochStore {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.clone())
    }
}
impl Debug for RoochStore {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl TransactionStore for RoochStore {
    fn save_transaction(&self, tx: LedgerTransaction) -> Result<()> {
        self.transaction_store.save_transaction(tx)
    }

    fn remove_transaction(&self, tx_hash: H256, tx_order: u64) -> Result<()> {
        self.transaction_store.remove_transaction(tx_hash, tx_order)
    }

    fn get_transaction_by_hash(&self, hash: H256) -> Result<Option<LedgerTransaction>> {
        self.transaction_store.get_transaction_by_hash(hash)
    }

    fn get_transactions_by_hash(
        &self,
        tx_hashes: Vec<H256>,
    ) -> Result<Vec<Option<LedgerTransaction>>> {
        self.transaction_store.get_transactions(tx_hashes)
    }

    fn get_tx_hashes(&self, tx_orders: Vec<u64>) -> Result<Vec<Option<H256>>> {
        self.transaction_store.get_tx_hashes(tx_orders)
    }
}

impl MetaStore for RoochStore {
    fn get_sequencer_info(&self) -> Result<Option<SequencerInfo>> {
        self.get_meta_store().get_sequencer_info()
    }

    fn save_sequencer_info(&self, sequencer_info: SequencerInfo) -> Result<()> {
        self.get_meta_store().save_sequencer_info(sequencer_info)
    }

    fn remove_sequencer_info(&self) -> Result<()> {
        self.get_meta_store().remove_sequence_info()
    }
}

impl StateStore for RoochStore {
    // Setting TTL directly in RocksDB may not be a good choice.
    // RocksDB uses compaction to remove expired keys,
    // and it may also have performance impact.
    // TODO Cleaning up data regularly may be an option
    fn save_state_change_set(
        &self,
        tx_order: u64,
        state_change_set: StateChangeSetExt,
    ) -> Result<()> {
        self.get_state_store()
            .save_state_change_set(tx_order, state_change_set)
    }

    fn get_state_change_set(&self, tx_order: u64) -> Result<Option<StateChangeSetExt>> {
        self.get_state_store().get_state_change_set(tx_order)
    }

    fn multi_get_state_change_set(
        &self,
        tx_orders: Vec<u64>,
    ) -> Result<Vec<Option<StateChangeSetExt>>> {
        self.get_state_store().multi_get_state_change_set(tx_orders)
    }

    fn remove_state_change_set(&self, tx_order: u64) -> Result<()> {
        self.get_state_store().remove_state_change_set(tx_order)
    }
}
