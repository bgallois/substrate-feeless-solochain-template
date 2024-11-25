use crate as pallet;
use frame_support::derive_impl;
use sp_runtime::{traits::parameter_types, BuildStorage};

type Balance = u64;
type BlockNumber = u64;
type Block = frame_system::mocking::MockBlock<Test>;

#[frame_support::runtime]
mod runtime {
    // The main runtime
    #[runtime::runtime]
    // Runtime Types to be generated
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason,
        RuntimeLockId,
        RuntimeTask
    )]
    pub struct Test;

    #[runtime::pallet_index(0)]
    pub type System = frame_system::Pallet<Test>;

    #[runtime::pallet_index(1)]
    pub type Feeless = pallet::Pallet<Test>;

    #[runtime::pallet_index(2)]
    pub type Balances = pallet_balances::Pallet<Test>;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type AccountData = pallet::AccountData<Balance, BlockNumber>;
    type Block = Block;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = Feeless;
    type Balance = Balance;
}

parameter_types! {
    pub const MaxTxByPeriod: u32 = 2;
    pub const Period: u32 = 10;
}

impl pallet::Config for Test {
    type MaxTxByPeriod = MaxTxByPeriod;
    type Period = Period;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}

/// A simple call, which one doesn't matter.
pub const CALL: &<Test as frame_system::Config>::RuntimeCall =
    &RuntimeCall::System(frame_system::Call::set_heap_pages { pages: 0u64 });