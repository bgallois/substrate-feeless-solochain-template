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
use crate as pallet;
use frame_support::derive_impl;
use frame_system::EnsureRoot;
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
    pub const MaxTxByPeriod: u32 = 5;
    pub const MaxSizeByPeriod: u32 = 40;
    pub const Period: u32 = 10;
}

impl pallet::Config for Test {
    type MaxSizeByPeriod = MaxSizeByPeriod;
    type MaxTxByPeriod = MaxTxByPeriod;
    type Period = Period;
    type RuntimeEvent = RuntimeEvent;
    type StatusOrigin = EnsureRoot<Self::AccountId>;
    type WeightInfo = ();
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
