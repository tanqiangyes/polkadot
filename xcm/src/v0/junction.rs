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

//! Support data structures for `MultiLocation`, primarily the `Junction` datatype.
//! 支持 `MultiLocation` 的数据结构，主要是 `Junction` 数据类型

use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

/// A global identifier of an account-bearing consensus system.
/// 记账共识系统的全局标识符。
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, TypeInfo)]
pub enum NetworkId {
	/// Unidentified/any.
	/// 无标识/全部
	Any,
	/// Some named network.
	/// 命名了的网络
	Named(Vec<u8>),
	/// The Polkadot Relay chain
	/// 波卡中继链
	Polkadot,
	/// Kusama.
	/// Kusama测试链
	Kusama,
}

/// An identifier of a pluralistic body.
/// 多元化id的标识符。
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, TypeInfo)]
pub enum BodyId {
	/// The only body in its context.
	/// 其上下文中唯一的主体。
	Unit,
	/// A named body.
	/// 命名的主体
	Named(Vec<u8>),
	/// An indexed body.
	/// 索引主体
	Index(#[codec(compact)] u32),
	/// The unambiguous executive body (for Polkadot, this would be the Polkadot council).
	/// 明确的执行机构（对于 Polkadot，这将是 Polkadot 委员会）。
	Executive,
	/// The unambiguous technical body (for Polkadot, this would be the Technical Committee).
	/// 明确的技术机构（对于 Polkadot，这将是技术委员会）。
	Technical,
	/// The unambiguous legislative body (for Polkadot, this could be considered the opinion of a majority of
	/// lock-voters).
	/// 明确的立法机构（对于 Polkadot，这可以被认为是大多数锁定选民的意见）。
	Legislative,
	/// The unambiguous judicial body (this doesn't exist on Polkadot, but if it were to get a "grand oracle", it
	/// may be considered as that).
	/// 明确的司法机构（在波卡上不存在，但如果要获得“大神谕”，可能会被认为是那样）。
	Judicial,
}

/// A part of a pluralistic body.
/// 多元化的body的一部分
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, TypeInfo)]
pub enum BodyPart {
	/// The body's declaration, under whatever means it decides.
	/// 身体的声明，无论以何种方式决定。
	Voice,
	/// A given number of members of the body.
	/// 给定数量的body成员。
	Members {
		#[codec(compact)]
		count: u32,
	},
	/// A given number of members of the body, out of some larger caucus.
	Fraction {
		#[codec(compact)]
		nom: u32,
		#[codec(compact)]
		denom: u32,
	},
	/// No less than the given proportion of members of the body.
	AtLeastProportion {
		#[codec(compact)]
		nom: u32,
		#[codec(compact)]
		denom: u32,
	},
	/// More than than the given proportion of members of the body.
	MoreThanProportion {
		#[codec(compact)]
		nom: u32,
		#[codec(compact)]
		denom: u32,
	},
}

impl BodyPart {
	/// Returns `true` if the part represents a strict majority (> 50%) of the body in question.
	/// 如果该部分代表所讨论的主体的严格多数 (> 50%)，则返回 `true`。
	pub fn is_majority(&self) -> bool {
		match self {
			BodyPart::Fraction { nom, denom } if *nom * 2 > *denom => true,
			BodyPart::AtLeastProportion { nom, denom } if *nom * 2 > *denom => true,
			BodyPart::MoreThanProportion { nom, denom } if *nom * 2 >= *denom => true,
			_ => false,
		}
	}
}

/// A single item in a path to describe the relative location of a consensus system.
/// 路径中的单个项目，用于描述共识系统的相对位置。
/// Each item assumes a pre-existing location as its context and is defined in terms of it.
/// 每个项目都假定一个预先存在的位置作为它的上下文，并根据它来定义。
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, TypeInfo)]
pub enum Junction {
	/// The consensus system of which the context is a member and state-wise super-set.
	/// 上下文是其成员和状态超集的共识系统。
	/// NOTE: This item is *not* a sub-consensus item: a consensus system may not identify itself trustlessly as
	/// a location that includes this junction.
	/// 注意：此项目不是子共识项目：共识系统可能不会不信任地将自己标识为包含此连接的位置。
	Parent,
	/// An indexed parachain belonging to and operated by the context.
	/// 属于上下文并由上下文操作的索引平行链。
	/// Generally used when the context is a Polkadot Relay-chain.
	/// 通常在上下文是 Polkadot 中继链时使用。
	Parachain(#[codec(compact)] u32),
	/// A 32-byte identifier for an account of a specific network that is respected as a sovereign endpoint within
	/// the context.
	/// 特定网络帐户的 32 字节标识符，在上下文中被视为主权端点。
	/// Generally used when the context is a Substrate-based chain.
	/// 通常在上下文是基于 Substrate 的链时使用。
	AccountId32 { network: NetworkId, id: [u8; 32] },
	/// An 8-byte index for an account of a specific network that is respected as a sovereign endpoint within
	/// the context.
	/// 特定网络帐户的 8 字节索引，在上下文中被视为主权端点。
	/// May be used when the context is a Frame-based chain and includes e.g. an indices pallet.
	/// 当上下文是基于帧的链并且包括例如一个索引托盘。
	AccountIndex64 {
		network: NetworkId,
		#[codec(compact)]
		index: u64,
	},
	/// A 20-byte identifier for an account of a specific network that is respected as a sovereign endpoint within
	/// the context.
	/// 特定网络帐户的 20 字节标识符，在上下文中被视为主权端点。
	/// May be used when the context is an Ethereum or Bitcoin chain or smart-contract.
	/// 当上下文是以太坊或比特币链或智能合约时可以使用。
	AccountKey20 { network: NetworkId, key: [u8; 20] },
	/// An instanced, indexed pallet that forms a constituent part of the context.
	/// 构成上下文组成部分的实例化索引托盘。
	/// Generally used when the context is a Frame-based chain.
	/// 通常在上下文是基于帧的链时使用。
	PalletInstance(u8),
	/// A non-descript index within the context location.
	/// 上下文位置中的非描述索引。
	/// Usage will vary widely owing to its generality.
	/// 由于其普遍性，用法将有很大差异。
	/// NOTE: Try to avoid using this and instead use a more specific item.
	/// 注意：尽量避免使用这个，而是使用更具体的项目。
	GeneralIndex(#[codec(compact)] u128),
	/// A nondescript datum acting as a key within the context location.
	/// 一个不起眼的数据，作为上下文位置中的一个键。
	/// Usage will vary widely owing to its generality.
	/// 由于其普遍性，用法将有很大差异。
	/// NOTE: Try to avoid using this and instead use a more specific item.
	/// 注意：尽量避免使用这个，而是使用更具体的项目。
	GeneralKey(Vec<u8>),
	/// The unambiguous child.
	/// 毫不含糊的孩子。
	/// Not currently used except as a fallback when deriving ancestry.
	/// 目前不使用，除非在派生祖先时作为后备。
	OnlyChild,
	/// A pluralistic body existing within consensus.
	/// 存在于共识中的多元化机构。
	/// Typical to be used to represent a governance origin of a chain, but could in principle be used to represent
	/// things such as multisigs also.
	/// 典型用于表示链的治理起源，但原则上也可用于表示诸如多重签名之类的事物。
	Plurality { id: BodyId, part: BodyPart },
}

impl From<crate::v1::Junction> for Junction {
	fn from(v1: crate::v1::Junction) -> Junction {
		use crate::v1::Junction::*;
		match v1 {
			Parachain(id) => Self::Parachain(id),
			AccountId32 { network, id } => Self::AccountId32 { network, id },
			AccountIndex64 { network, index } => Self::AccountIndex64 { network, index },
			AccountKey20 { network, key } => Self::AccountKey20 { network, key },
			PalletInstance(index) => Self::PalletInstance(index),
			GeneralIndex(index) => Self::GeneralIndex(index),
			GeneralKey(key) => Self::GeneralKey(key),
			OnlyChild => Self::OnlyChild,
			Plurality { id, part } => Self::Plurality { id, part },
		}
	}
}

impl Junction {
	/// Returns true if this junction is a `Parent` item.
	pub fn is_parent(&self) -> bool {
		match self {
			Junction::Parent => true,
			_ => false,
		}
	}

	/// Returns true if this junction can be considered an interior part of its context. This is generally `true`,
	/// except for the `Parent` item.
	/// 如果此联结可被视为其上下文的内部部分，则返回 true。这通常是 `true`，除了 `Parent` 项。
	pub fn is_interior(&self) -> bool {
		match self {
			Junction::Parent => false,

			Junction::Parachain(..) |
			Junction::AccountId32 { .. } |
			Junction::AccountIndex64 { .. } |
			Junction::AccountKey20 { .. } |
			Junction::PalletInstance { .. } |
			Junction::GeneralIndex { .. } |
			Junction::GeneralKey(..) |
			Junction::OnlyChild |
			Junction::Plurality { .. } => true,
		}
	}
}
