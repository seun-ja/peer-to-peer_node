use std::fmt::Debug;

use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use sha256::digest;

use crate::utils::UnixTimestamp;

pub type TransactionHash = String;
pub type BlockHash = String;

/// Represents a block.
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

/// Represents the header of a block.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockHeader {
    pub previous_block_hash: BlockHash,
    pub timestamp: UnixTimestamp,
    pub nonce: String,
    pub transaction_hash: TransactionHash,
}

/// Represents a transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    _signature: Option<String>,
    pub(crate) _timestamp: UnixTimestamp,
    pub(crate) _transaction: TransactionData,
}

impl Transaction {
    /// Hashes the transaction in sha256 digest
    fn _to_hash(&self) -> Result<String, TransactionError> {
        serde_json::to_string(&self)
            .map_err(TransactionError::UnableToSerializeTransaction)
            .map(digest)
    }

    /// Encodes the transaction in hex format
    pub fn to_hex(&self) -> Result<String, TransactionError> {
        serde_json::to_string(&self)
            .map_err(TransactionError::UnableToDeserializeTransaction)
            .map(hex::encode)
    }
}

/// Transaction data. Ideally contains payload that's `Transactable`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionData {
    _transaction_id: String,
    _payload: Box<dyn Transactable>,
}

pub struct BlockMetadata {}

#[typetag::serde(tag = "type")]
trait Transactable: Debug + Send + Sync + DynClone {
    // TODO: Implement the _submit method to submit the transaction to the network.
    fn _submit(&self) -> Result<(), TransactionError>;
}

dyn_clone::clone_trait_object!(Transactable);

#[derive(thiserror::Error, Debug)]
pub enum TransactionError {
    #[error("Unable to serialize transaction: {0}")]
    UnableToSerializeTransaction(serde_json::Error),
    #[error("Unable to deserialize transaction: {0}")]
    UnableToDeserializeTransaction(serde_json::Error),
}
