use super::*;
use crate::Pallet;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks(
    where
        T: frame_system::Config<AccountData = AccountData<T::Balance, BlockNumberFor<T>>>
            + Config
            + pallet_balances::Config,)
]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn set_status() {
        let caller: T::AccountId = whitelisted_caller();
        frame_system::Account::<T>::mutate(&caller, |_| {}); // Init AccountData

        #[extrinsic_call]
        _(RawOrigin::Root, caller.clone(), crate::Status::Unlimited);

        assert_eq!(
            frame_system::Account::<T>::get(&caller).data.rate.status,
            crate::Status::Unlimited
        );
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
