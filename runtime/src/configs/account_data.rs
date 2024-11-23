use codec::{Decode, Encode, MaxEncodedLen};
use core::marker::PhantomData;
use frame_support::pallet_prelude::InvalidTransaction::ExhaustsResources;
use frame_support::pallet_prelude::InvalidTransaction::UnknownOrigin;
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_runtime::traits::DispatchInfoOf;
use sp_runtime::traits::Dispatchable;
use sp_runtime::traits::PostDispatchInfoOf;
use sp_runtime::transaction_validity::TransactionSource;
use sp_runtime::transaction_validity::TransactionValidityError;
use sp_runtime::transaction_validity::ValidTransaction;
use sp_runtime::DispatchError;
use sp_runtime::DispatchResult;
use sp_runtime::RuntimeDebug;
use sp_runtime::SaturatedConversion;
use sp_runtime::Weight;
use sp_runtime::{impl_tx_ext_default, traits::TransactionExtension};

const MAX_TX_BY_PERIOD: u32 = 2;
const PERIOD: u32 = 5;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Rate<BlockNumber> {
    pub last_block: BlockNumber,
    pub tx_since_last: u32,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AccountData<Balance, BlockNumber> {
    pub balance: pallet_balances::AccountData<Balance>,
    pub rate: Rate<BlockNumber>,
}

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

trait RateLimiter<BlockNumber> {
    fn is_allowed(&self, b: BlockNumber) -> bool;
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

impl<T: frame_system::Config + Send + Sync> TransactionExtension<T::RuntimeCall> for CheckRate<T>
where
    T::AccountData: RateLimiter<BlockNumberFor<T>>,
{
    const IDENTIFIER: &'static str = "CheckRate";
    type Implicit = ();
    type Pre = Pre<T>;
    type Val = Pre<T>;

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

    impl_tx_ext_default!(T::RuntimeCall; weight);
}
