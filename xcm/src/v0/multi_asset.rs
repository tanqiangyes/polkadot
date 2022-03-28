// Copyright 2020 Parity Technologies (UK) Ltd.
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

//! Cross-Consensus Message format data structures.
//! 交叉共识消息格式数据结构。

use super::MultiLocation;
use crate::v1::{MultiAssetFilter, MultiAssets, WildMultiAsset};
use alloc::{vec, vec::Vec};
use core::{
	convert::{TryFrom, TryInto},
	result,
};
use parity_scale_codec::{self, Decode, Encode};
use scale_info::TypeInfo;

pub use crate::v1::AssetInstance;

/// A single general identifier for an asset.
/// 资产的单一通用标识符。
/// Represents both fungible and non-fungible assets. May only be used to represent a single asset class.
/// 代表可替代和不可替代的资产。只能用于表示单个资产类别。
/// Wildcards may or may not be allowed by the interpreting context.
/// 解释上下文可能允许也可能不允许通配符。
/// Assets classes may be identified in one of two ways: either an abstract identifier or a concrete identifier.
/// Implementations may support only one of these. A single asset may be referenced from multiple asset identifiers,
/// though will tend to have only a single *preferred* identifier.
/// 资产类别可以通过以下两种方式之一进行标识：抽象标识符或具体标识符。实现可能仅支持其中之一。单个资产可以从多个资产标识符中引用，但往往只有一个首选标识符。
/// ### Abstract identifiers
///
/// Abstract identifiers are absolute identifiers that represent a notional asset which can exist within multiple
/// consensus systems. These tend to be simpler to deal with since their broad meaning is unchanged regardless stay of
/// the consensus system in which it is interpreted.
/// 抽象标识符是代表可以存在于多个共识系统中的名义资产的绝对标识符。这些往往更容易处理，因为无论解释它的共识系统是否停留，它们的广泛含义都不会改变。
/// However, in the attempt to provide uniformity across consensus systems, they may conflate different instantiations
/// of some notional asset (e.g. the reserve asset and a local reserve-backed derivative of it) under the same name,
/// leading to confusion. It also implies that one notional asset is accounted for locally in only one way. This may not
/// be the case, e.g. where there are multiple bridge instances each providing a bridged "BTC" token yet none being
/// fungible between the others.
/// 然而，为了在共识系统之间提供统一性，他们可能会将某些名义资产（例如储备资产和它的本地储备支持的衍生品）的不同实例混为一谈，从而导致混淆。
/// 这也意味着一种名义资产仅以一种方式在当地进行核算。情况可能并非如此，例如其中有多个桥接实例，每个实例都提供一个桥接的“BTC”令牌，但彼此之间没有可替代的。
/// Since they are meant to be absolute and universal, a global registry is needed to ensure that name collisions do not
/// occur.
/// 由于它们是绝对的和通用的，因此需要一个全局注册表来确保不会发生名称冲突。
/// An abstract identifier is represented as a simple variable-size byte string. As of writing, no global registry
/// exists and no proposals have been put forth for asset labeling.
/// 抽象标识符表示为简单的可变大小字节字符串。在撰写本文时，不存在全球注册机构，也没有提出资产标签提案。
/// ### Concrete identifiers
///
/// Concrete identifiers are *relative identifiers* that specifically identify a single asset through its location in a
/// consensus system relative to the context interpreting. Use of a `MultiLocation` ensures that similar but non
/// fungible variants of the same underlying asset can be properly distinguished, and obviates the need for any kind of
/// central registry.
/// 具体标识符是相对标识符，通过其在共识系统中相对于上下文解释的位置来专门识别单个资产。
/// 使用“MultiLocation”可确保可以正确区分相同基础资产的相似但不可替代的变体，并且无需任何类型的中央注册表。
/// The limitation is that the asset identifier cannot be trivially copied between consensus systems and must instead be
/// "re-anchored" whenever being moved to a new consensus system, using the two systems' relative paths.
/// 限制是资产标识符不能在共识系统之间简单地复制，而是必须在移动到新的共识系统时使用两个系统的相对路径“重新锚定”。
/// Throughout XCM, messages are authored such that *when interpreted from the receiver's point of view* they will have
/// the desired meaning/effect. This means that relative paths should always by constructed to be read from the point of
/// view of the receiving system, *which may be have a completely different meaning in the authoring system*.
/// 在整个 XCM 中，消息的编写使得当从接收者的角度解释时，它们将具有所需的意义效果。这意味着相对路径应始终构造为从接收系统的角度读取，这在创作系统中可能具有完全不同的含义。
/// Concrete identifiers are the preferred way of identifying an asset since they are entirely unambiguous.
/// 具体标识符是识别资产的首选方式，因为它们是完全明确的。
/// A concrete identifier is represented by a `MultiLocation`. If a system has an unambiguous primary asset (such as
/// Bitcoin with BTC or Ethereum with ETH), then it will conventionally be identified as the chain itself. Alternative
/// and more specific ways of referring to an asset within a system include:
/// 具体标识符由“MultiLocation”表示。如果一个系统具有明确的主要资产（例如比特币与 BTC 或以太坊与 ETH），那么它通常会被识别为链本身。
/// 在系统中引用资产的替代和更具体的方法包括：
/// - `<chain>/PalletInstance(<id>)` for a Frame chain with a single-asset pallet instance (such as an instance of the
///   Balances pallet).
/// - `<chain>/PalletInstance(<id>)/GeneralIndex(<index>)` for a Frame chain with an indexed multi-asset pallet instance
///   (such as an instance of the Assets pallet).
/// - `<chain>/AccountId32` for an ERC-20-style single-asset smart-contract on a Frame-based contracts chain.
/// - `<chain>/AccountKey20` for an ERC-20-style single-asset smart-contract on an Ethereum-like chain.
/// - `<chain>PalletInstance(<id>)` 用于具有单一资产托盘实例（例如 Balances 托盘实例）的框架链。
/// - `<chain>PalletInstance(<id>)GeneralIndex(<index>)` 用于具有索引多资产托盘实例（例如资产托盘实例）的框架链。
/// - `<chain>AccountId32` 用于基于框架的合约链上的 ERC-20 风格的单一资产智能合约。
/// - `<chain>AccountKey20` 用于类似以太坊的链上的 ERC-20 风格的单一资产智能合约。
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, TypeInfo)]
pub enum MultiAsset {
	/// No assets. Rarely used.
	/// 没有资产。很少使用。
	None,

	/// All assets. Typically used for the subset of assets to be used for an `Order`, and in that context means
	/// "all assets currently in holding".
	/// 所有资产。通常用于用于“Order”的资产子集，在该上下文中表示“当前持有的所有资产”。
	All,

	/// All fungible assets. Typically used for the subset of assets to be used for an `Order`, and in that context
	/// means "all fungible assets currently in holding".
	/// 所有可替代资产。通常用于用于“Order”的资产子集，在该上下文中表示“当前持有的所有可替代资产”。
	AllFungible,

	/// All non-fungible assets. Typically used for the subset of assets to be used for an `Order`, and in that
	/// context means "all non-fungible assets currently in holding".
	/// 所有不可替代的资产。通常用于用于“Order”的资产子集，在该上下文中表示“当前持有的所有不可替代资产”。
	AllNonFungible,

	/// All fungible assets of a given abstract asset `id`entifier.
	/// 给定抽象资产`id`entifier的所有可替代资产。
	AllAbstractFungible { id: Vec<u8> },

	/// All non-fungible assets of a given abstract asset `class`.
	/// 给定抽象资产“类别”的所有不可替代资产。
	AllAbstractNonFungible { class: Vec<u8> },

	/// All fungible assets of a given concrete asset `id`entifier.
	/// 给定具体资产“标识符”的所有可替代资产。
	AllConcreteFungible { id: MultiLocation },

	/// All non-fungible assets of a given concrete asset `class`.
	/// 给定具体资产“类别”的所有不可替代资产。
	AllConcreteNonFungible { class: MultiLocation },

	/// Some specific `amount` of the fungible asset identified by an abstract `id`.
	/// 由抽象“id”标识的可替代资产的某些特定“数量”。
	AbstractFungible {
		id: Vec<u8>,
		#[codec(compact)]
		amount: u128,
	},

	/// Some specific `instance` of the non-fungible asset whose `class` is identified abstractly.
	/// 不可替代资产的某些特定“实例”，其“类别”被抽象识别。
	AbstractNonFungible { class: Vec<u8>, instance: AssetInstance },

	/// Some specific `amount` of the fungible asset identified by an concrete `id`.
	/// 由具体“id”标识的可替代资产的某些特定“amount”。
	ConcreteFungible {
		id: MultiLocation,
		#[codec(compact)]
		amount: u128,
	},

	/// Some specific `instance` of the non-fungible asset whose `class` is identified concretely.
	/// 不可替代资产的某些特定“实例”，其“类别”被具体识别。
	ConcreteNonFungible { class: MultiLocation, instance: AssetInstance },
}

impl MultiAsset {
	/// Returns `true` if the `MultiAsset` is a wildcard and can refer to classes of assets, instead of just one.
	/// 如果 `MultiAsset` 是通配符并且可以引用资产类别，而不仅仅是一个，则返回 `true`。
	/// Typically can also be inferred by the name starting with `All`.
	/// 通常也可以通过以“All”开头的名称来推断。
	pub fn is_wildcard(&self) -> bool {
		match self {
			MultiAsset::None |
			MultiAsset::AbstractFungible { .. } |
			MultiAsset::AbstractNonFungible { .. } |
			MultiAsset::ConcreteFungible { .. } |
			MultiAsset::ConcreteNonFungible { .. } => false,

			MultiAsset::All |
			MultiAsset::AllFungible |
			MultiAsset::AllNonFungible |
			MultiAsset::AllAbstractFungible { .. } |
			MultiAsset::AllConcreteFungible { .. } |
			MultiAsset::AllAbstractNonFungible { .. } |
			MultiAsset::AllConcreteNonFungible { .. } => true,
		}
	}

	fn is_none(&self) -> bool {
		match self {
			MultiAsset::None |
			MultiAsset::AbstractFungible { amount: 0, .. } |
			MultiAsset::ConcreteFungible { amount: 0, .. } => true,

			_ => false,
		}
	}

	fn is_fungible(&self) -> bool {
		match self {
			MultiAsset::All |
			MultiAsset::AllFungible |
			MultiAsset::AllAbstractFungible { .. } |
			MultiAsset::AllConcreteFungible { .. } |
			MultiAsset::AbstractFungible { .. } |
			MultiAsset::ConcreteFungible { .. } => true,

			_ => false,
		}
	}

	fn is_non_fungible(&self) -> bool {
		match self {
			MultiAsset::All |
			MultiAsset::AllNonFungible |
			MultiAsset::AllAbstractNonFungible { .. } |
			MultiAsset::AllConcreteNonFungible { .. } |
			MultiAsset::AbstractNonFungible { .. } |
			MultiAsset::ConcreteNonFungible { .. } => true,

			_ => false,
		}
	}

	fn is_concrete_fungible(&self, id: &MultiLocation) -> bool {
		match self {
			MultiAsset::AllFungible => true,
			MultiAsset::AllConcreteFungible { id: i } |
			MultiAsset::ConcreteFungible { id: i, .. } => i == id,

			_ => false,
		}
	}

	fn is_abstract_fungible(&self, id: &[u8]) -> bool {
		match self {
			MultiAsset::AllFungible => true,
			MultiAsset::AllAbstractFungible { id: i } |
			MultiAsset::AbstractFungible { id: i, .. } => i == id,
			_ => false,
		}
	}

	fn is_concrete_non_fungible(&self, class: &MultiLocation) -> bool {
		match self {
			MultiAsset::AllNonFungible => true,
			MultiAsset::AllConcreteNonFungible { class: i } |
			MultiAsset::ConcreteNonFungible { class: i, .. } => i == class,
			_ => false,
		}
	}

	fn is_abstract_non_fungible(&self, class: &[u8]) -> bool {
		match self {
			MultiAsset::AllNonFungible => true,
			MultiAsset::AllAbstractNonFungible { class: i } |
			MultiAsset::AbstractNonFungible { class: i, .. } => i == class,
			_ => false,
		}
	}

	fn is_all(&self) -> bool {
		matches!(self, MultiAsset::All)
	}

	/// Returns true if `self` is a super-set of the given `inner`.
	/// 如果 `self` 是给定 `inner` 的超集，则返回 true。
	/// Typically, any wildcard is never contained in anything else, and a wildcard can contain any other non-wildcard.
	/// For more details, see the implementation and tests.
	pub fn contains(&self, inner: &MultiAsset) -> bool {
		use MultiAsset::*;

		// Inner cannot be wild
		if inner.is_wildcard() {
			return false
		}
		// Everything contains nothing.
		if inner.is_none() {
			return true
		}

		// Everything contains anything.
		if self.is_all() {
			return true
		}
		// Nothing contains nothing.
		if self.is_none() {
			return false
		}

		match self {
			// Anything fungible contains "all fungibles"
			AllFungible => inner.is_fungible(),
			// Anything non-fungible contains "all non-fungibles"
			AllNonFungible => inner.is_non_fungible(),

			AllConcreteFungible { id } => inner.is_concrete_fungible(id),
			AllAbstractFungible { id } => inner.is_abstract_fungible(id),
			AllConcreteNonFungible { class } => inner.is_concrete_non_fungible(class),
			AllAbstractNonFungible { class } => inner.is_abstract_non_fungible(class),

			ConcreteFungible { id, amount } => matches!(
				inner,
				ConcreteFungible { id: inner_id , amount: inner_amount } if inner_id == id && amount >= inner_amount
			),
			AbstractFungible { id, amount } => matches!(
				inner,
				AbstractFungible { id: inner_id , amount: inner_amount } if inner_id == id && amount >= inner_amount
			),
			ConcreteNonFungible { .. } => self == inner,
			AbstractNonFungible { .. } => self == inner,
			_ => false,
		}
	}

	pub fn reanchor(&mut self, prepend: &MultiLocation) -> Result<(), ()> {
		use MultiAsset::*;
		match self {
			AllConcreteFungible { ref mut id } |
			AllConcreteNonFungible { class: ref mut id } |
			ConcreteFungible { ref mut id, .. } |
			ConcreteNonFungible { class: ref mut id, .. } =>
				id.prepend_with(prepend.clone()).map_err(|_| ()),
			_ => Ok(()),
		}
	}
}

impl TryFrom<crate::v1::MultiAsset> for MultiAsset {
	type Error = ();

	fn try_from(m: crate::v1::MultiAsset) -> result::Result<MultiAsset, ()> {
		use crate::v1::{AssetId::*, Fungibility::*};
		use MultiAsset::*;
		Ok(match (m.id, m.fun) {
			(Concrete(id), Fungible(amount)) => ConcreteFungible { id: id.try_into()?, amount },
			(Concrete(class), NonFungible(instance)) =>
				ConcreteNonFungible { class: class.try_into()?, instance },
			(Abstract(id), Fungible(amount)) => AbstractFungible { id, amount },
			(Abstract(class), NonFungible(instance)) => AbstractNonFungible { class, instance },
		})
	}
}

impl TryFrom<MultiAssets> for Vec<MultiAsset> {
	type Error = ();

	fn try_from(m: MultiAssets) -> result::Result<Vec<MultiAsset>, ()> {
		m.drain().into_iter().map(MultiAsset::try_from).collect()
	}
}

impl TryFrom<WildMultiAsset> for MultiAsset {
	type Error = ();

	fn try_from(m: WildMultiAsset) -> result::Result<MultiAsset, ()> {
		use crate::v1::{AssetId::*, WildFungibility::*};
		use MultiAsset::*;
		Ok(match m {
			WildMultiAsset::All => All,
			WildMultiAsset::AllOf { id, fun } => match (id, fun) {
				(Concrete(id), Fungible) => AllConcreteFungible { id: id.try_into()? },
				(Concrete(class), NonFungible) =>
					AllConcreteNonFungible { class: class.try_into()? },
				(Abstract(id), Fungible) => AllAbstractFungible { id },
				(Abstract(class), NonFungible) => AllAbstractNonFungible { class },
			},
		})
	}
}

impl TryFrom<WildMultiAsset> for Vec<MultiAsset> {
	type Error = ();

	fn try_from(m: WildMultiAsset) -> result::Result<Vec<MultiAsset>, ()> {
		Ok(vec![m.try_into()?])
	}
}

impl TryFrom<MultiAssetFilter> for Vec<MultiAsset> {
	type Error = ();

	fn try_from(m: MultiAssetFilter) -> result::Result<Vec<MultiAsset>, ()> {
		match m {
			MultiAssetFilter::Definite(assets) => assets.try_into(),
			MultiAssetFilter::Wild(wildcard) => wildcard.try_into(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn contains_works() {
		use alloc::vec;
		use MultiAsset::*;
		// trivial case: all contains any non-wildcard.
		assert!(All.contains(&None));
		assert!(All.contains(&AbstractFungible { id: alloc::vec![99u8], amount: 1 }));

		// trivial case: none contains nothing, except itself.
		assert!(None.contains(&None));
		assert!(!None.contains(&AllFungible));
		assert!(!None.contains(&All));

		// A bit more sneaky: Nothing can contain wildcard, even All ir the thing itself.
		assert!(!All.contains(&All));
		assert!(!All.contains(&AllFungible));
		assert!(!AllFungible.contains(&AllFungible));
		assert!(!AllNonFungible.contains(&AllNonFungible));

		// For fungibles, containing is basically equality, or equal id with higher amount.
		assert!(!AbstractFungible { id: vec![99u8], amount: 99 }
			.contains(&AbstractFungible { id: vec![1u8], amount: 99 }));
		assert!(AbstractFungible { id: vec![99u8], amount: 99 }
			.contains(&AbstractFungible { id: vec![99u8], amount: 99 }));
		assert!(AbstractFungible { id: vec![99u8], amount: 99 }
			.contains(&AbstractFungible { id: vec![99u8], amount: 9 }));
		assert!(!AbstractFungible { id: vec![99u8], amount: 99 }
			.contains(&AbstractFungible { id: vec![99u8], amount: 100 }));

		// For non-fungibles, containing is equality.
		assert!(!AbstractNonFungible { class: vec![99u8], instance: AssetInstance::Index(9) }
			.contains(&AbstractNonFungible {
				class: vec![98u8],
				instance: AssetInstance::Index(9)
			}));
		assert!(!AbstractNonFungible { class: vec![99u8], instance: AssetInstance::Index(8) }
			.contains(&AbstractNonFungible {
				class: vec![99u8],
				instance: AssetInstance::Index(9)
			}));
		assert!(AbstractNonFungible { class: vec![99u8], instance: AssetInstance::Index(9) }
			.contains(&AbstractNonFungible {
				class: vec![99u8],
				instance: AssetInstance::Index(9)
			}));
	}
}
