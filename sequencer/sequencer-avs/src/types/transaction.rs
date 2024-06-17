use super::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserTransaction {
    encrypted_transaction: EncryptedTransaction,
    time_lock_puzzle: TimeLockPuzzle,
    nonce: Nonce,
}

impl AsRef<[u8]> for UserTransaction {
    fn as_ref(&self) -> &[u8] {
        self.encrypted_transaction.as_ref()
    }
}

impl UserTransaction {
    const ID: &'static str = stringify!(Transaction);

    pub fn get(
        rollup_block_number: RollupBlockNumber,
        transaction_order: u64,
    ) -> Result<Self, database::Error> {
        let key = (Self::ID, rollup_block_number, transaction_order);
        database()?.get(&key)
    }

    pub fn put(
        &self,
        rollup_block_number: RollupBlockNumber,
        transaction_order: u64,
    ) -> Result<(), database::Error> {
        let key = (Self::ID, rollup_block_number, transaction_order);
        database()?.put(&key, self)
    }

    pub fn new(
        encrypted_transaction: EncryptedTransaction,
        time_lock_puzzle: TimeLockPuzzle,
        nonce: Nonce,
    ) -> Self {
        Self {
            encrypted_transaction,
            time_lock_puzzle,
            nonce,
        }
    }

    pub fn encrypted_transaction(&self) -> &EncryptedTransaction {
        &self.encrypted_transaction
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EncryptedTransaction(String);

impl AsRef<[u8]> for EncryptedTransaction {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl EncryptedTransaction {
    pub fn new(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().to_owned())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TimeLockPuzzle {
    t: u8,
    g: String,
    n: String,
}

impl TimeLockPuzzle {
    pub fn new(t: u8, g: impl AsRef<str>, n: impl AsRef<str>) -> Self {
        Self {
            t,
            g: g.as_ref().to_owned(),
            n: n.as_ref().to_owned(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Nonce(String);

impl Nonce {
    pub fn new(value: impl AsRef<str>) -> Self {
        Self(value.as_ref().to_owned())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderCommitment {
    rollup_block_number: RollupBlockNumber,
    transaction_order: u64,
}

impl OrderCommitment {
    pub fn new(rollup_block_number: RollupBlockNumber, transaction_order: u64) -> Self {
        Self {
            rollup_block_number,
            transaction_order,
        }
    }

    pub fn rollup_block_number(&self) -> RollupBlockNumber {
        self.rollup_block_number
    }

    pub fn transaction_order(&self) -> u64 {
        self.transaction_order
    }
}
