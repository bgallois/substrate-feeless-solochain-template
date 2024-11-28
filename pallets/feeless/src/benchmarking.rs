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
