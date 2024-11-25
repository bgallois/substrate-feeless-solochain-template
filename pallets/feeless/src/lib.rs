#![cfg_attr(not(feature = "std"), no_std)]

/// # Rate Limiting System with `CheckRate` Transaction Extension For Feeless Chain
///
/// This pallet implements a rate-limiting mechanism for Substrate-based blockchains.
/// It enforces limits on the number of transactions an account can make within a specified block period.
///
/// ## Overview
///
/// - Accounts are limited to a maximum number of transactions (`MaxTxByPeriod`) within
///   a specified block duration (`Period`).
/// - The mechanism tracks transaction rates using the `Rate` struct, stored within the account's
///   custom `AccountData`.
/// - Validation and rate updates occur during the transaction lifecycle (validation, preparation,
///   and post-dispatch phases).
///
/// ## Key Concepts
///
/// - **Rate Limiting**: Controlled through the `Rate` struct, which tracks the last processed block
///   and the number of transactions since that block.
/// - **Transaction Extension**: Implemented through the `CheckRate` struct, integrated with the
///   Substrate transaction validation pipeline.
///
/// ## Integration into the Runtime
/// In the runtime configuration, we specify the following settings to enable the rate-limiting mechanism:
///
/// ### 1. `AccountData` Definition
/// ```rust,ignore
/// type AccountData = pallet_feeless::AccountData<Balance, BlockNumber>;
/// ```
/// The `AccountData` struct is used to store the balance, last block, and transaction count for each account.
///
/// ### 2. Account Store in `pallet_balances::Config`
/// ```rust,ignore
/// type AccountStore = Account;
/// ```
///
/// ### 3. Transaction Extension (`TxExtension`)
/// In the Runtime (lib.rs):
/// ```rust,ignore
/// pub type TxExtension = (
///     frame_system::CheckNonZeroSender<Runtime>,
///     frame_system::CheckSpecVersion<Runtime>,
///     frame_system::CheckTxVersion<Runtime>,
///     frame_system::CheckGenesis<Runtime>,
///     frame_system::CheckEra<Runtime>,
///     frame_system::CheckNonce<Runtime>,
///     pallet_feeless::CheckRate<Runtime>, // Add CheckRate for rate-limiting validation
///     frame_system::CheckWeight<Runtime>,
///     pallet_transaction_payment::ChargeTransactionPayment<Runtime>, // Remove fees
///     frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
/// );
///
///
/// impl pallet_transaction_payment::Config for Runtime {
///     type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
///     type LengthToFee = IdentityFee<Balance>;
///     type OnChargeTransaction = FungibleAdapter<Balances, ()>;
///     type OperationalFeeMultiplier = ConstU8<5>;
///     type RuntimeEvent = RuntimeEvent;
///     type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
///     type WeightToFee = frame_support::weights::FixedFee<0, Balance>;
/// }
/// ```
/// This extends the transaction validation with a rate check to ensure the account isn't exceeding its transaction limit.
///
/// In the Node (benchmarkings.rs):
/// ```rust,ignore
/// let tx_ext: runtime::TxExtension = (
///     frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
///     frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
///     frame_system::CheckTxVersion::<runtime::Runtime>::new(),
///     frame_system::CheckGenesis::<runtime::Runtime>::new(),
///     frame_system::CheckEra::<runtime::Runtime>::from(sp_runtime::generic::Era::mortal(
///         period,
///         best_block.saturated_into(),
///     )),
///     frame_system::CheckNonce::<runtime::Runtime>::from(nonce),
///     pallet_feeless::CheckRate::<runtime::Runtime>::new(),
///     frame_system::CheckWeight::<runtime::Runtime>::new(),
///     pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0),
///     frame_metadata_hash_extension::CheckMetadataHash::<runtime::Runtime>::new(false),
/// );
///
/// let raw_payload = runtime::SignedPayload::from_raw(
///     call.clone(),
///     tx_ext.clone(),
///     (
///         (),
///         runtime::VERSION.spec_version,
///         runtime::VERSION.transaction_version,
///         genesis_hash,
///         best_hash,
///         (),
///         (),
///         (),
///         (),
///         None,
///     ),
/// );
/// ```
///
use frame_support::traits::Get;
use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet::*;
use sp_runtime::{DispatchError, SaturatedConversion};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod types;
pub use types::*;

pub mod extensions;
pub use extensions::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Maximum number of transactions allowed per account within the defined period.
        type MaxTxByPeriod: Get<u32>;
        /// Duration (in blocks) defining the rate-limiting period.
        type Period: Get<u32>;
    }
}

/// Implements the storage backend for custom account data (same as the default from pallet
/// balances.
impl<T> frame_support::traits::StoredMap<T::AccountId, pallet_balances::AccountData<T::Balance>>
    for Pallet<T>
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

impl<T> RateLimiter<T> for AccountData<T::Balance, BlockNumberFor<T>>
where
    T: frame_system::Config<AccountData = AccountData<T::Balance, BlockNumberFor<T>>>
        + Config
        + pallet_balances::Config,
{
    fn is_allowed(&self, b: BlockNumberFor<T>) -> bool {
        (b - self.rate.last_block).saturated_into::<u32>() >= T::Period::get()
            || self.rate.tx_since_last < T::MaxTxByPeriod::get()
    }

    fn update_rate(&mut self, b: BlockNumberFor<T>) {
        if (b - self.rate.last_block).saturated_into::<u32>() < T::Period::get() {
            self.rate.tx_since_last += 1;
        } else {
            self.rate.tx_since_last = 1;
            self.rate.last_block = b;
        }
    }
}