use codec::{Decode, Encode, MaxEncodedLen};
use core::marker::PhantomData;
use frame_support::pallet_prelude::MaybeSerializeDeserialize;
use frame_support::Parameter;
use scale_info::TypeInfo;
use sp_runtime::traits::Debug;
use sp_runtime::traits::Member;
use sp_runtime::DispatchError;
use sp_runtime::RuntimeDebug;

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

pub struct AccountStore<T, AccountId, Balance, BlockNumber>(
    PhantomData<T>,
    PhantomData<AccountId>,
    PhantomData<Balance>,
    PhantomData<BlockNumber>,
);
impl<T, AccountId, Balance, BlockNumber>
    frame_support::traits::StoredMap<AccountId, pallet_balances::AccountData<Balance>>
    for AccountStore<T, AccountId, Balance, BlockNumber>
where
    AccountId: Parameter
        + Member
        + MaybeSerializeDeserialize
        + Debug
        + sp_runtime::traits::MaybeDisplay
        + Ord
        + Into<[u8; 32]>
        + codec::MaxEncodedLen,
    Balance: frame_support::traits::tokens::Balance,
    BlockNumber: sp_runtime::traits::BlockNumber,
    T: frame_system::Config<AccountId = AccountId, AccountData = AccountData<Balance, BlockNumber>>
        + pallet_balances::Config<Balance = Balance>,
{
    fn get(k: &AccountId) -> pallet_balances::AccountData<Balance> {
        frame_system::Account::<T>::get(k).data.balance
    }

    fn try_mutate_exists<R, E: From<DispatchError>>(
        k: &AccountId,
        f: impl FnOnce(&mut Option<pallet_balances::AccountData<Balance>>) -> Result<R, E>,
    ) -> Result<R, E> {
        let account = frame_system::Account::<T>::get(k);
        let is_default = account.data.balance == pallet_balances::AccountData::<Balance>::default();
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
