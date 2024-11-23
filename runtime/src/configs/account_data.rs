//! # Rate Limiting System with `CheckRate` Transaction Extension
//!
//! This module implements a rate-limiting mechanism for Substrate-based blockchains.
//! It defines a transaction extension (`CheckRate`) that enforces limits on the number of
//! transactions an account can make in a defined block period.
//!
//! ## Overview
//!
//! - Accounts are limited to a maximum number of transactions (`MAX_TX_BY_PERIOD`) within
//!   a specified block duration (`PERIOD`).
//! - The mechanism tracks transaction rates using the `Rate` struct, stored within the account's
//!   custom `AccountData`.
//! - Validation and rate updates occur during the transaction lifecycle (validation, preparation,
//!   and post-dispatch phases).
//!
//! ## Key Concepts
//!
//! - **Rate Limiting**: Controlled through the `Rate` struct, which tracks the last processed block
//!   and the number of transactions since that block.
//! - **Transaction Extension**: Implemented through the `CheckRate` struct, integrated with the
//!   Substrate transaction validation pipeline.

use codec::{Decode, Encode, MaxEncodedLen};
use core::marker::PhantomData;
use frame_support::pallet_prelude::InvalidTransaction::{ExhaustsResources, UnknownOrigin};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_runtime::{
    impl_tx_ext_default,
    traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf, TransactionExtension},
    transaction_validity::{TransactionSource, TransactionValidityError, ValidTransaction},
    DispatchError, DispatchResult, RuntimeDebug, SaturatedConversion, Weight,
};

/// Maximum number of transactions allowed per account within the defined period.
const MAX_TX_BY_PERIOD: u32 = 2;

/// Duration (in blocks) defining the rate-limiting period.
const PERIOD: u32 = 5;

/// Tracks transaction rates for an account over blocks.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Rate<BlockNumber> {
    /// Block number of the last transaction.
    pub last_block: BlockNumber,
    /// Number of transactions since the last block.
    pub tx_since_last: u32,
}

/// Custom account data structure with rate limiting.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AccountData<Balance, BlockNumber> {
    /// Balance data from the `pallet_balances` module.
    pub balance: pallet_balances::AccountData<Balance>,
    /// Rate limiter data.
    pub rate: Rate<BlockNumber>,
}

/// Implements the storage backend for custom account data (same as the default from pallet
/// balances.
pub struct AccountStore<T>(PhantomData<T>);
impl<T> frame_support::traits::StoredMap<T::AccountId, pallet_balances::AccountData<T::Balance>>
    for AccountStore<T>
where
    T: frame_system::Config<AccountData = AccountData<T::Balance, BlockNumberFor<T>>>
        + pallet_balances::Config,
{
    fn get(k: &T::AccountId) -> pallet_balances::AccountData<T::Balance> {
        frame_system::Account::<T>::get(k).data.balance
    }

    fn try_mutate_exists<R, E: From<DispatchError>>(
        k: &T::AccountId,
        f: impl FnOnce(&mut Option<pallet_balances::AccountData<T::Balance>>) -> Result<R, E>,
    ) -> Result<R, E> {
        let account = frame_system::Account::<T>::get(k);
        let is_default =
            account.data.balance == pallet_balances::AccountData::<T::Balance>::default();
        let mut some_data = if is_default {
            None
        } else {
            Some(account.data.balance)
        };
        let result = f(&mut some_data)?;
        if frame_system::Pallet::<T>::providers(k) > 0
            || frame_system::Pallet::<T>::sufficients(k) > 0
        {
            frame_system::Account::<T>::mutate(k, |a| {
                a.data.balance = some_data.unwrap_or_default()
            });
        } else {
            frame_system::Account::<T>::remove(k)
        }
        Ok(result)
    }
}

/// Rate-limiting behavior.
trait RateLimiter<BlockNumber> {
    /// Checks if a transaction is allowed for the current block.
    fn is_allowed(&self, b: BlockNumber) -> bool;
    /// Updates the rate limiter after a transaction.
    fn update_rate(&mut self, b: BlockNumber);
}

impl<BlockNumber, Balance> RateLimiter<BlockNumber> for AccountData<Balance, BlockNumber>
where
    Balance: frame_support::traits::tokens::Balance,
    BlockNumber: sp_runtime::traits::BlockNumber,
{
    fn is_allowed(&self, b: BlockNumber) -> bool {
        (b - self.rate.last_block).saturated_into::<u32>() >= PERIOD
            || self.rate.tx_since_last < MAX_TX_BY_PERIOD
    }

    fn update_rate(&mut self, b: BlockNumber) {
        if (b - self.rate.last_block).saturated_into::<u32>() < PERIOD {
            self.rate.tx_since_last += 1;
        } else {
            self.rate.tx_since_last = 1;
            self.rate.last_block = b;
        }
    }
}

/// A transaction extension for rate limiting.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CheckRate<T: frame_system::Config + Send + Sync>(PhantomData<T>);

impl<T: frame_system::Config + Send + Sync> core::fmt::Debug for CheckRate<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "CheckRate")
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result {
        Ok(())
    }
}

pub struct Pre<T: frame_system::Config> {
    who: T::AccountId,
}

impl<T: frame_system::Config> core::fmt::Debug for Pre<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "who: {:?}", self.who)
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("<wasm:stripped>")
    }
}

impl<T: frame_system::Config + Send + Sync> Default for CheckRate<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: frame_system::Config + Send + Sync> CheckRate<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> TransactionExtension<T::RuntimeCall> for CheckRate<T>
where
    T: frame_system::Config + Send + Sync,
    T::AccountData: RateLimiter<BlockNumberFor<T>>,
{
    type Implicit = ();
    type Pre = Pre<T>;
    type Val = Pre<T>;

    const IDENTIFIER: &'static str = "CheckRate";

    impl_tx_ext_default!(T::RuntimeCall; weight);

    /// Validates a transaction based on rate limits.
    fn validate(
        &self,
        origin: <T::RuntimeCall as Dispatchable>::RuntimeOrigin,
        _call: &T::RuntimeCall,
        _info: &DispatchInfoOf<T::RuntimeCall>,
        _len: usize,
        _: (),
        _implication: &impl Encode,
        _source: TransactionSource,
    ) -> Result<
        (
            ValidTransaction,
            Self::Val,
            <T::RuntimeCall as Dispatchable>::RuntimeOrigin,
        ),
        TransactionValidityError,
    > {
        let Ok(who) = frame_system::ensure_signed(origin.clone()) else {
            return Err(TransactionValidityError::Invalid(UnknownOrigin));
        };
        let account_data = frame_system::Account::<T>::get(who.clone()).data;
        let block = frame_system::Pallet::<T>::block_number();
        if account_data.is_allowed(block) {
            Ok((Default::default(), Pre { who: who.clone() }, origin))
        } else {
            Err(TransactionValidityError::Invalid(ExhaustsResources))
        }
    }

    /// Prepares data for post-dispatch processing.
    fn prepare(
        self,
        val: Self::Val,
        _origin: &<T::RuntimeCall as Dispatchable>::RuntimeOrigin,
        _call: &T::RuntimeCall,
        _info: &DispatchInfoOf<T::RuntimeCall>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        Ok(val)
    }

    /// Updates rate limits after transaction execution.
    fn post_dispatch_details(
        pre: Self::Pre,
        _info: &DispatchInfoOf<T::RuntimeCall>,
        _post_info: &PostDispatchInfoOf<T::RuntimeCall>,
        _len: usize,
        _result: &DispatchResult,
    ) -> Result<Weight, TransactionValidityError> {
        let mut account_data = frame_system::Account::<T>::get(pre.who.clone()).data;
        let block = frame_system::Pallet::<T>::block_number();
        account_data.update_rate(block);
        frame_system::Account::<T>::mutate(pre.who, |account| account.data = account_data);
        Ok(Weight::zero())
    }
}
