use std::io;

use super::Transaction;

use crate::serialization::{sha256d, ZcashSerialize};

use super::Hash;

pub(super) struct TxIdHasher<'a> {
    trans: &'a Transaction,
}

impl<'a> TxIdHasher<'a> {
    pub fn new(trans: &'a Transaction) -> Self {
        TxIdHasher { trans }
    }

    pub(super) fn txid(self) -> Result<Hash, io::Error> {
        match self.trans {
            Transaction::V1 { .. }
            | Transaction::V2 { .. }
            | Transaction::V3 { .. }
            | Transaction::V4 { .. } => self.txid_v4(),
            Transaction::V5 { .. } => self.txid_v5(),
        }
    }

    fn txid_v4(self) -> Result<Hash, io::Error> {
        let mut hash_writer = sha256d::Writer::default();
        self.trans
            .zcash_serialize(&mut hash_writer)
            .expect("Transactions must serialize into the hash.");
        Ok(Hash(hash_writer.finish()))
    }

    fn txid_v5(self) -> Result<Hash, io::Error> {
        // The v5 txid (from ZIP-244) is computed using librustzcash. Convert the zebra
        // transaction to a librustzcash transaction.

        let serialized_tx = self.trans.zcash_serialize_to_vec()?;
        // The `read` method ignores the passed BranchId for V5 transactions;
        // we arbitrarly pass `Nu5`.
        let alt_tx = zcash_primitives::transaction::Transaction::read(
            &serialized_tx[..],
            zcash_primitives::consensus::BranchId::Nu5,
        )?;

        Ok(Hash(*alt_tx.txid().as_ref()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{serialization::ZcashDeserializeInto, transaction::Transaction};
    use color_eyre::eyre;
    use eyre::Result;
    use zebra_test::zip0244::TEST_VECTORS;

    #[test]
    fn txid() -> Result<()> {
        zebra_test::init();

        // TODO: we don't support Orchard deserialization yet; so test it only
        // with a fixed transaction that does not use it
        for test in TEST_VECTORS[0..1].iter() {
            let transaction = test.tx.zcash_deserialize_into::<Transaction>()?;
            let hasher = TxIdHasher::new(&transaction);
            let txid = hasher.txid()?;
            assert_eq!(txid.0, test.txid);
        }

        Ok(())
    }
}
