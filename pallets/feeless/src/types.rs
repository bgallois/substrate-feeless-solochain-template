use codec::{Decode, Encode, MaxEncodedLen};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// Tracks transaction rates for an account over blocks.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Rate<BlockNumber> {
    /// Block number of the last transaction.
    pub last_block: BlockNumber,
    /// Number of transactions since the last block.
    pub tx_since_last: u32,
    /// Size of transactions since the last block.
    pub size_since_last: u32,
}

/// Custom account data structure with rate limiting.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AccountData<Balance, BlockNumber> {
    /// Balance data from the `pallet_balances` module.
    pub balance: pallet_balances::AccountData<Balance>,
    /// Rate limiter data.
    pub rate: Rate<BlockNumber>,
}

/// Rate-limiting behavior.
pub trait RateLimiter<T: frame_system::Config> {
    /// Checks if a transaction is allowed for the current block.
    fn is_allowed(&self, b: BlockNumberFor<T>, size: u32) -> bool;
    /// Updates the rate limiter after a transaction.
    fn update_rate(&mut self, b: BlockNumberFor<T>, size: u32);
}
