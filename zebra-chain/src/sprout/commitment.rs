//! Sprout commitment types.

#![allow(clippy::unit_arg)]

use sha2::{Digest, Sha256};

use super::note::Note;

/// The randomness used in the Pedersen Hash for note commitment.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct CommitmentRandomness(pub [u8; 32]);

impl AsRef<[u8]> for CommitmentRandomness {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Note commitments for the output notes.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct NoteCommitment(pub(crate) [u8; 32]);

impl Eq for NoteCommitment {}

impl From<[u8; 32]> for NoteCommitment {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<Note> for NoteCommitment {
    /// NoteCommit_rcm^Sprout(a_pk, v, rho)
    ///
    /// https://zips.z.cash/protocol/protocol.pdf#concretesproutnotecommit
    fn from(note: Note) -> NoteCommitment {
        let leading_byte: u8 = 0xB0;
        let mut hasher = Sha256::default();
        hasher.input([leading_byte]);
        hasher.input(note.paying_key);
        hasher.input(note.value.to_bytes());
        hasher.input(note.rho);
        hasher.input(note.rcm);
        NoteCommitment(hasher.result().into())
    }
}

impl From<NoteCommitment> for [u8; 32] {
    fn from(cm: NoteCommitment) -> [u8; 32] {
        cm.0
    }
}