 // Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

pub use self::currency::DOLLARS;

/// Money matters.
/// 单位
pub mod currency {
	use primitives::v2::Balance;

	/// The existential deposit.
	pub const EXISTENTIAL_DEPOSIT: Balance = 100 * CENTS;

	pub const UNITS: Balance = 10_000_000_000;
	pub const DOLLARS: Balance = UNITS; // 10_000_000_000
	pub const CENTS: Balance = DOLLARS / 100; // 100_000_000
	pub const MILLICENTS: Balance = CENTS / 1_000; // 100_000

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 20 * DOLLARS + (bytes as Balance) * 100 * MILLICENTS
	}
}

/// Time and blocks.
/// 时间和块
pub mod time {
	use primitives::v2::{BlockNumber, Moment};
	pub const MILLISECS_PER_BLOCK: Moment = 6000; //6s一个块
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;//插槽间隔
	pub const EPOCH_DURATION_IN_SLOTS: BlockNumber = 4 * HOURS;//插槽的周期

	// These time units are defined in number of blocks.
	// 这些时间单位以块数定义。
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);//分钟
	pub const HOURS: BlockNumber = MINUTES * 60;//小时
	pub const DAYS: BlockNumber = HOURS * 24;//一天
	pub const WEEKS: BlockNumber = DAYS * 7;//一周

	// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
}

/// Fee-related.
/// 费用相关。
pub mod fee {
	use crate::weights::ExtrinsicBaseWeight;
	use frame_support::weights::{
		WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
	};
	use primitives::v2::Balance;
	use smallvec::smallvec;
	pub use sp_runtime::Perbill;

	/// The block saturation level. Fees will be updates based on this value.
	/// 块饱和度。费用将根据此值更新。
	pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///	根据节点余额类型的规模和粒度，处理将权重标量转换为费用值。
	/// This should typically create a mapping between the following ranges:
	///   - [0, `MAXIMUM_BLOCK_WEIGHT`]
	///   - [Balance::min, Balance::max]
	///	这通常应该在以下范围之间创建映射： - [0, `MAXIMUM_BLOCK_WEIGHT`] - [Balance::min, Balance::max]
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	/// 然而，它可以用于任何其他类型的重量费变化。一些示例是：
	/// - 将其设置为“0”将基本上禁用重量费。
	/// - 将其设置为 `1` 将导致对字面 `#[weight = x]` 值进行收费。
	pub struct WeightToFee;
	impl WeightToFeePolynomial for WeightToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// in Polkadot, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
			// 在 Polkadot 中，外部基础权重（最小的非零权重）映射到 1/10 CENT：
			let p = super::currency::CENTS;
			let q = 10 * Balance::from(ExtrinsicBaseWeight::get());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{
		currency::{CENTS, DOLLARS, MILLICENTS},
		fee::WeightToFee,
	};
	use crate::weights::ExtrinsicBaseWeight;
	use frame_support::weights::WeightToFeePolynomial;
	use runtime_common::MAXIMUM_BLOCK_WEIGHT;

	#[test]
	// This function tests that the fee for `MAXIMUM_BLOCK_WEIGHT` of weight is correct
	fn full_block_fee_is_correct() {
		// A full block should cost 16 DOLLARS
		println!("Base: {}", ExtrinsicBaseWeight::get());
		let x = WeightToFee::calc(&MAXIMUM_BLOCK_WEIGHT);
		let y = 16 * DOLLARS;
		assert!(x.max(y) - x.min(y) < MILLICENTS);
	}

	#[test]
	// This function tests that the fee for `ExtrinsicBaseWeight` of weight is correct
	fn extrinsic_base_fee_is_correct() {
		// `ExtrinsicBaseWeight` should cost 1/10 of a CENT
		println!("Base: {}", ExtrinsicBaseWeight::get());
		let x = WeightToFee::calc(&ExtrinsicBaseWeight::get());
		let y = CENTS / 10;
		assert!(x.max(y) - x.min(y) < MILLICENTS);
	}
}
