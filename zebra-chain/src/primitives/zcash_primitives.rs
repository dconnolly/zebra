//! Contains code that interfaces with the zcash_primitives crate from
//! librustzcash.

use std::{
    convert::{TryFrom, TryInto},
    io,
};

use crate::{
    amount::{Amount, NonNegative},
    serialization::ZcashSerialize,
    transaction::HashType,
    transaction::Transaction,
    transparent::{self, Script},
};

impl TryFrom<&Transaction> for zcash_primitives::transaction::Transaction {
    type Error = io::Error;

    /// Convert a Zebra transaction into a librustzcash one.
    ///
    /// # Panics
    ///
    /// If the transaction is not V5. (Currently there is no need for this
    /// conversion for other versions.)
    fn try_from(trans: &Transaction) -> Result<Self, Self::Error> {
        let network_upgrade = match trans {
            Transaction::V5 {
                network_upgrade, ..
            } => network_upgrade,
            Transaction::V1 { .. }
            | Transaction::V2 { .. }
            | Transaction::V3 { .. }
            | Transaction::V4 { .. } => panic!("Zebra only uses librustzcash for V5 transactions"),
        };

        let serialized_tx = trans.zcash_serialize_to_vec()?;
        // The `read` method currently ignores the BranchId for V5 transactions;
        // but we use the correct BranchId anyway.
        let branch_id: u32 = network_upgrade
            .branch_id()
            .expect("Network upgrade must have a Branch ID")
            .into();
        // We've already parsed this transaction, so its network upgrade must be valid.
        let branch_id: zcash_primitives::consensus::BranchId = branch_id
            .try_into()
            .expect("zcash_primitives and Zebra have the same branch ids");
        let alt_tx =
            zcash_primitives::transaction::Transaction::read(&serialized_tx[..], branch_id)?;
        Ok(alt_tx)
    }
}

/// Convert a Zebra Amount into a librustzcash one.
impl TryFrom<Amount<NonNegative>> for zcash_primitives::transaction::components::Amount {
    type Error = ();

    fn try_from(amount: Amount<NonNegative>) -> Result<Self, Self::Error> {
        zcash_primitives::transaction::components::Amount::from_u64(amount.into())
    }
}

/// Convert a Zebra Script into a librustzcash one.
impl From<&Script> for zcash_primitives::legacy::Script {
    fn from(script: &Script) -> Self {
        zcash_primitives::legacy::Script(script.0.clone())
    }
}

/// Compute a signature hash for a V5 transaction using librustzcash.
///
/// # Inputs
///
/// - `transaction`: the transaction whose signature hash to compute.
/// - `hash_type`: the type of hash (SIGHASH) being used.
/// - `input`: information about the transparent input for which this signature
///     hash is being computed, if any. A tuple with the matching output of the
///     previous transaction, the input itself, and the index of the input in
///     the transaction.
pub(crate) fn sighash_v5(
    trans: &Transaction,
    hash_type: &HashType,
    input: Option<(&transparent::Output, &transparent::Input, usize)>,
) -> blake2b_simd::Hash {
    let alt_tx: zcash_primitives::transaction::Transaction = trans
        .try_into()
        .expect("zcash_primitives and Zebra transaction formats must be compatible");

    let script: zcash_primitives::legacy::Script;
    let signable_input = match input {
        Some((output, _, idx)) => {
            script = (&output.lock_script).into();
            zcash_primitives::transaction::sighash::SignableInput::Transparent(
                zcash_primitives::transaction::sighash::TransparentInput::new(
                    idx,
                    &script,
                    output
                        .value
                        .try_into()
                        .expect("amount was previously validated"),
                ),
            )
        }
        None => zcash_primitives::transaction::sighash::SignableInput::Shielded,
    };

    alt_tx.sighash(signable_input, hash_type.bits()).into()
}
