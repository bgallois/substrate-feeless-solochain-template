// GNU General Public License (GPL)
// Version 3, 29 June 2007
// http://www.gnu.org/licenses/gpl-3.0.html
//
// Copyright 2024 Benjamin Gallois
//
// Licensed under the GNU General Public License, Version 3 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.gnu.org/licenses/gpl-3.0.html
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// You may not distribute modified versions of the software without providing
// the source code, and any derivative works must be licensed under the GPL
// License as well. This ensures that the software remains free and open
// for all users.
//
// You should have received a copy of the GPL along with this program.
// If not, see <http://www.gnu.org/licenses/>.
#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]
use frame_support::{pallet_prelude::IsType, traits::Get};
use frame_system::{
    ensure_root,
    pallet_prelude::{BlockNumberFor, OriginFor},
};
pub use pallet::*;
use sp_runtime::{DispatchError, DispatchResult, SaturatedConversion};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

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
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Maximum number of transactions allowed per account within the defined period.
        type MaxTxByPeriod: Get<u32>;
        /// Maximum size of transactions allowed per account within the defined period.
        type MaxSizeByPeriod: Get<u32>;
        /// Duration (in blocks) defining the rate-limiting period.
        type Period: Get<u32>;
        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        StatusChanged { who: T::AccountId, status: Status },
    }

    #[pallet::error]
    pub enum Error<T> {
        StatusNotChanged,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        T: frame_system::Config<AccountData = AccountData<T::Balance, BlockNumberFor<T>>>
            + Config
            + pallet_balances::Config,
    {
        /// Sets the status of a specific account.
        ///
        /// This function allows the root user to update the status of an account.
        /// It is typically used for management tasks, such as managing account states
        /// during runtime upgrades or other administrative actions.
        ///
        /// The status of the account will be updated to the provided `status` value.
        ///
        /// ## Arguments:
        /// - `origin`: The origin of the transaction (must be the root account).
        /// - `who`: The `AccountId` of the account whose status is being set.
        /// - `status`: The new `Status` to assign to the account.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::set_status())]
        pub fn set_status(
            origin: OriginFor<T>,
            who: T::AccountId,
            status: Status,
        ) -> DispatchResult {
            ensure_root(origin)?;

            Self::deposit_event(Event::StatusChanged {
                who: who.clone(),
                status: status.clone(),
            });
            frame_system::Account::<T>::try_mutate_exists(who.clone(), |account| {
                if let Some(ref mut account) = account {
                    account.data.rate.status = status.clone();
                    Ok(())
                } else {
                    Err(Error::<T>::StatusNotChanged.into())
                }
            })
        }
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

/// A rate limiter implementation for managing transaction limits
/// and data size constraints based on a specified block number period.
///
/// This implementation limits the number of transactions and the total
/// size of transactions that can be processed within a given period. The
/// rate limiter checks whether the rate limit has been exceeded, and updates
/// the rate statistics accordingly.
impl<T> RateLimiter<T> for AccountData<T::Balance, BlockNumberFor<T>>
where
    T: frame_system::Config<AccountData = AccountData<T::Balance, BlockNumberFor<T>>>
        + Config
        + pallet_balances::Config,
{
    /// Determines whether a transaction is allowed based on the current rate
    /// limiter settings, considering the block number and the transaction size.
    ///
    /// # Arguments
    /// * `b` - The current block number.
    /// * `len` - The size of the transaction in bytes.
    ///
    /// # Returns
    /// `true` if the transaction is allowed, `false` otherwise.
    fn is_allowed(&self, b: BlockNumberFor<T>, len: u32) -> bool {
        if self.rate.status == Status::Unlimited {
            true
        } else if (b - self.rate.last_block).saturated_into::<u32>() < T::Period::get() {
            self.rate.tx_since_last < T::MaxTxByPeriod::get()
                && self.rate.size_since_last.saturating_add(len) < T::MaxSizeByPeriod::get()
        } else {
            len < T::MaxSizeByPeriod::get()
        }
    }

    /// Updates the rate limiter's internal statistics, such as the number of
    /// transactions and the total data size for the current period, based on
    /// the current block number and transaction size.
    ///
    /// # Arguments
    /// * `b` - The current block number.
    /// * `len` - The size of the transaction in bytes.
    ///
    /// This method will reset the transaction count and size if the current
    /// block number exceeds the specified period. Otherwise, it will update
    /// the transaction count and size based on the new transaction.
    fn update_rate(&mut self, b: BlockNumberFor<T>, len: u32) {
        if (b - self.rate.last_block).saturated_into::<u32>() < T::Period::get() {
            self.rate.tx_since_last += 1;
            self.rate.size_since_last += len;
        } else {
            self.rate.tx_since_last = 1;
            self.rate.size_since_last = len;
            self.rate.last_block = b;
        }
    }
}
