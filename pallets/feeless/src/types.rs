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
use codec::{Decode, Encode, MaxEncodedLen};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Status {
    #[default]
    Limited,
    Unlimited,
}

/// Tracks transaction rates for an account over blocks.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Rate<BlockNumber> {
    /// Block number of the last transaction.
    pub last_block: BlockNumber,
    /// Number of transactions since the last block.
    pub tx_since_last: u32,
    /// Size of transactions since the last block.
    pub size_since_last: u32,
    pub status: Status,
}

/// Custom account data structure with rate limiting.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AccountData<Balance, BlockNumber> {
    /// Balance data from the `pallet_balances` module.
    pub balance: pallet_balances::AccountData<Balance>,
    /// Rate limiter data.
    pub rate: Rate<BlockNumber>,
}

/// Rate-limiting behavior.
pub trait RateLimiter<T: frame_system::Config> {
    /// Checks if a transaction is allowed for the current block.
    fn is_allowed(&self, b: BlockNumberFor<T>, size: u32) -> bool;
    /// Updates the rate limiter after a transaction.
    fn update_rate(&mut self, b: BlockNumberFor<T>, size: u32);
}
