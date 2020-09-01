use crate::meta_addr::MetaAddr;
use std::sync::Arc;
use zebra_chain::block::{self, Block};

/// A response to a network request, represented in internal format.
#[derive(Clone, Debug)]
pub enum Response {
    /// A response with no data.
    Nil,

    /// A list of peers, used to respond to `GetPeers`.
    Peers(Vec<MetaAddr>),

    /// A list of blocks.
    Blocks(Vec<Arc<Block>>),

    /// A list of block hashes.
    BlockHashes(Vec<block::Hash>),
}
