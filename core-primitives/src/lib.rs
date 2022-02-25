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

#![cfg_attr(not(feature = "std"), no_std)]

//! Core Polkadot types.
//!
//! These core Polkadot types are used by the relay chain and the Parachains.

use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use parity_util_mem::MallocSizeOf;
use scale_info::TypeInfo;
use sp_runtime::{
	generic,
	traits::{IdentifyAccount, Verify},
	MultiSignature,
};

pub use sp_runtime::traits::{BlakeTwo256, Hash as HashT};

/// The block number type used by Polkadot.
/// 32-bits will allow for 136 years of blocks assuming 1 block per second.
/// 波卡使用的区块号类型。假设每秒 1 个块，32 位将允许 136 年的块。
pub type BlockNumber = u32;

/// An instant or duration in time.
/// 时间的瞬间或持续时间。
pub type Moment = u64;

/// Alias to type for a signature for a transaction on the relay chain. This allows one of several
/// kinds of underlying crypto to be used, so isn't a fixed size when encoded.
/// 为中继链上的交易签名键入的别名。这允许使用几种底层加密中的一种，因此在编码时不是固定大小。
pub type Signature = MultiSignature;

/// Alias to the public key used for this chain, actually a `MultiSigner`. Like the signature, this
/// also isn't a fixed size when encoded, as different cryptos have different size public keys.
/// 用于此链的公钥的别名，实际上是“MultiSigner”。与签名一样，这在编码时也不是固定大小，因为不同的密码具有不同大小的公钥。
pub type AccountPublic = <Signature as Verify>::Signer;

/// Alias to the opaque account ID type for this chain, actually a `AccountId32`. This is always
/// 32 bytes.
/// 此链的不透明帐户 ID 类型的别名，实际上是 `AccountId32`。这始终是 32 字节。
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
/// 用于查找帐户的类型。我们预计其中的数量不会超过 40 亿。
pub type AccountIndex = u32;

/// Identifier for a chain. 32-bit should be plenty.
/// 链的标识符。 32位应该足够了。
pub type ChainId = u32;

/// A hash of some data used by the relay chain.
/// 中继链使用的一些数据的哈希。
pub type Hash = sp_core::H256;

/// Unit type wrapper around [`Hash`] that represents a candidate hash.
///	[`Hash`] 周围的单元类型包装器，表示候选哈希。
/// This type is produced by [`CandidateReceipt::hash`].
///	此类型由 [`CandidateReceipt::hash`] 生成。
/// This type makes it easy to enforce that a hash is a candidate hash on the type level.
/// 这种类型可以很容易地执行，哈希是类型级别的候选哈希。
#[derive(Clone, Copy, Encode, Decode, Hash, Eq, PartialEq, Default, PartialOrd, Ord, TypeInfo)]
#[cfg_attr(feature = "std", derive(MallocSizeOf))]
pub struct CandidateHash(pub Hash);

#[cfg(feature = "std")]
impl std::ops::Deref for CandidateHash {
	type Target = Hash;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[cfg(feature = "std")]
impl std::fmt::Display for CandidateHash {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl sp_std::fmt::Debug for CandidateHash {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		write!(f, "{:?}", self.0)
	}
}

/// Index of a transaction in the relay chain. 32-bit should be plenty.
/// 中继链中交易的索引。 32位应该足够了。
pub type Nonce = u32;

/// The balance of an account.
/// 128-bits (or 38 significant decimal figures) will allow for 10 m currency (`10^7`) at a resolution
/// to all for one second's worth of an annualised 50% reward be paid to a unit holder (`10^11` unit
/// denomination), or `10^18` total atomic units, to grow at 50%/year for 51 years (`10^9` multiplier)
/// for an eventual total of `10^27` units (27 significant decimal figures).
/// We round denomination to `10^12` (12 SDF), and leave the other redundancy at the upper end so
/// that 32 bits may be multiplied with a balance in 128 bits without worrying about overflow.
/// 账户余额。 128 位（或 38 位有效十进制数字）将允许 1000 万货币（`10^7`）在决议中向所有人支付一秒钟的年化 50% 奖励给单位持有人（`10^11`单位面额）或“10^18”总原子单位，
/// 以 50% 的年增长率持续 51 年（“10^9”乘数），最终总计“10^27”单位（27 个有效小数）。我们将面额四舍五入为“10^12”（12 SDF），并将其他冗余留在上端，
/// 这样 32 位可以乘以 128 位的余额，而不必担心溢出。
pub type Balance = u128;

/// Header type.
/// 头类型
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
/// 区块类型
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// Block ID.
/// 区块id
pub type BlockId = generic::BlockId<Block>;

/// Opaque, encoded, unchecked extrinsic.
/// 不透明的、编码的、未经检查的外在的。
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

/// The information that goes alongside a `transfer_into_parachain` operation. Entirely opaque, it
/// will generally be used for identifying the reason for the transfer. Typically it will hold the
/// destination account to which the transfer should be credited. If still more information is
/// needed, then this should be a hash with the pre-image presented via an off-chain mechanism on
/// the parachain.
/// 与“transfer_into_parachain”操作一起出现的信息。完全不透明，它通常用于识别转移的原因。
/// 通常，它将持有应记入转账的目标账户。如果还需要更多信息，那么这应该是通过平行链上的链下机制呈现的具有原像的哈希。
pub type Remark = [u8; 32];

/// A message sent from the relay-chain down to a parachain.
/// 从中继链向下发送到平行链的消息
/// The size of the message is limited by the `config.max_downward_message_size` parameter.
/// 消息的大小受 `config.max_downward_message_size` 参数限制。
pub type DownwardMessage = sp_std::vec::Vec<u8>;

/// A wrapped version of `DownwardMessage`. The difference is that it has attached the block number when
/// the message was sent.
/// `DownwardMessage` 的包装版本。不同之处在于它在发送消息时附加了块号。
#[derive(Encode, Decode, Clone, sp_runtime::RuntimeDebug, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive(MallocSizeOf))]
pub struct InboundDownwardMessage<BlockNumber = crate::BlockNumber> {
	/// The block number at which these messages were put into the downward message queue.
	/// 这些是消息被放入向下消息队列的块号。
	pub sent_at: BlockNumber,
	/// The actual downward message to processes.
	pub msg: DownwardMessage,
}

/// An HRMP message seen from the perspective of a recipient.
/// 从收件人的角度看到的 HRMP 消息。
#[derive(Encode, Decode, Clone, sp_runtime::RuntimeDebug, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive(MallocSizeOf))]
pub struct InboundHrmpMessage<BlockNumber = crate::BlockNumber> {
	/// The block number at which this message was sent.
	/// Specifically, it is the block number at which the candidate that sends this message was
	/// enacted.
	/// 发送此消息的块号。具体来说，它是发送此消息的候选者所在的区块号。
	pub sent_at: BlockNumber,
	/// The message payload.
	/// 消息有效负载。
	pub data: sp_std::vec::Vec<u8>,
}

/// An HRMP message seen from the perspective of a sender.
/// 从发件人的角度看到的 HRMP 消息。
#[derive(Encode, Decode, Clone, sp_runtime::RuntimeDebug, PartialEq, Eq, Hash, TypeInfo)]
#[cfg_attr(feature = "std", derive(MallocSizeOf))]
pub struct OutboundHrmpMessage<Id> {
	/// The para that will get this message in its downward message queue.
	pub recipient: Id,
	/// The message payload.
	pub data: sp_std::vec::Vec<u8>,
}

/// `V2` primitives.
pub mod v2 {
	pub use super::*;
}
