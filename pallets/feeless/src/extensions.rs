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
use crate::types::RateLimiter;
use codec::{Decode, DecodeWithMemTracking, Encode};
use core::marker::PhantomData;
use frame_support::pallet_prelude::InvalidTransaction::ExhaustsResources;
use scale_info::TypeInfo;
use sp_runtime::{
    impl_tx_ext_default,
    traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf, TransactionExtension},
    transaction_validity::{TransactionSource, TransactionValidityError, ValidTransaction},
    DispatchResult, Weight,
};

/// A transaction extension for rate limiting.
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
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
    who: Option<T::AccountId>,
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
    T::AccountData: RateLimiter<T>,
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
        len: usize,
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
            return Ok((Default::default(), Pre { who: None }, origin));
        };

        let account_data = frame_system::Account::<T>::get(who.clone()).data;
        let block = frame_system::Pallet::<T>::block_number();
        if account_data.is_allowed(block, len as u32) {
            Ok((
                Default::default(),
                Pre {
                    who: Some(who.clone()),
                },
                origin,
            ))
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
        len: usize,
        _result: &DispatchResult,
    ) -> Result<Weight, TransactionValidityError> {
        if let Some(who) = pre.who {
            let mut account_data = frame_system::Account::<T>::get(who.clone()).data;
            let block = frame_system::Pallet::<T>::block_number();
            account_data.update_rate(block, len as u32);
            frame_system::Account::<T>::mutate(who, |account| account.data = account_data);
        }
        Ok(Weight::zero())
    }
}
