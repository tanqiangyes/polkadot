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

//! Version 0 of the Cross-Consensus Message format data structures.
//! 交叉共识消息格式数据结构的版本 0。

use crate::DoubleEncoded;
use alloc::vec::Vec;
use core::{
	convert::{TryFrom, TryInto},
	result,
};
use derivative::Derivative;
use parity_scale_codec::{self, Decode, Encode};
use scale_info::TypeInfo;

mod junction;
mod multi_asset;
mod multi_location;
mod order;
mod traits;
use super::v1::{MultiLocation as MultiLocation1, Response as Response1, Xcm as Xcm1};
pub use junction::{BodyId, BodyPart, Junction, NetworkId};
pub use multi_asset::{AssetInstance, MultiAsset};
pub use multi_location::MultiLocation::{self, *};
pub use order::Order;
pub use traits::{Error, ExecuteXcm, Outcome, Result, SendXcm};

/// A prelude for importing all types typically used when interacting with XCM messages.
/// 导入与 XCM 消息交互时通常使用的所有类型的前奏。
pub mod prelude {
	pub use super::{
		junction::{BodyId, Junction::*},
		multi_asset::{
			AssetInstance::{self, *},
			MultiAsset::{self, *},
		},
		multi_location::MultiLocation::{self, *},
		order::Order::{self, *},
		traits::{Error as XcmError, ExecuteXcm, Outcome, Result as XcmResult, SendXcm},
		Junction::*,
		OriginKind,
		Xcm::{self, *},
	};
}

// TODO: #2841 #XCMENCODE Efficient encodings for MultiAssets, Vec<Order>, using initial byte values 128+ to encode
//   the number of items in the vector.

/// Basically just the XCM (more general) version of `ParachainDispatchOrigin`.
/// 基本上只是 `ParachainDispatchOrigin` 的 XCM（更通用）版本。
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo)]
pub enum OriginKind {
	/// Origin should just be the native dispatch origin representation for the sender in the
	/// local runtime framework. For Cumulus/Frame chains this is the `Parachain` or `Relay` origin
	/// if coming from a chain, though there may be others if the `MultiLocation` XCM origin has a
	/// primary/native dispatch origin form.
	///  Origin应该只是本地运行时框架中发送者的本地调度起源表示。
	/// 对于Cumulus/Frame链，如果来自一个链，这就是 "Parachain "或 "Relay "原点，
	/// 不过如果 "MultiLocation "XCM原点有一个主要/本地调度原点形式，可能还有其他原点。
	Native,

	/// Origin should just be the standard account-based origin with the sovereign account of
	/// the sender. For Cumulus/Frame chains, this is the `Signed` origin.
	/// 原点应该只是标准的基于账户的原点，有发件人的主权账户。对于Cumulus/Frame链来说，这就是 "签名 "原点。
	SovereignAccount,

	/// Origin should be the super-user. For Cumulus/Frame chains, this is the `Root` origin.
	/// This will not usually be an available option.
	/// 原点应该是超级用户。对于Cumulus/Frame链，这是 "Root "起源。 这通常不会是一个可用的选项。
	Superuser,

	/// Origin should be interpreted as an XCM native origin and the `MultiLocation` should be
	/// encoded directly in the dispatch origin unchanged. For Cumulus/Frame chains, this will be
	/// the `pallet_xcm::Origin::Xcm` type.
	/// Origin 应该被解释为一个 XCM 原生的 origin，并且 `MultiLocation` 应该直接在 dispatch origin 中编码而不改变。
	/// 对于 Cumulus/Frame 链，这将是 `pallet_xcm::Origin::Xcm` 类型。
	Xcm,
}

/// Response data to a query.
/// 对查询的响应数据。
#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo)]
pub enum Response {
	/// Some assets.
	/// 一些资产。
	Assets(Vec<MultiAsset>),
}

/// Cross-Consensus Message: A message from one consensus system to another.
/// 交叉共识消息：从一个共识系统到另一个共识系统的消息。
/// Consensus systems that may send and receive messages include blockchains and smart contracts.
/// 可以发送和接收消息的共识系统包括区块链和智能合约。
/// All messages are delivered from a known *origin*, expressed as a `MultiLocation`.
/// 所有消息都从已知*origin*传递，表示为“MultiLocation”。
/// This is the inner XCM format and is version-sensitive. Messages are typically passed using the outer
/// XCM format, known as `VersionedXcm`.
/// 这是内部 XCM 格式并且是版本敏感的。消息通常使用外部 XCM 格式传递，称为 `VersionedXcm`。
#[derive(Derivative, Encode, Decode, TypeInfo)]
#[derivative(Clone(bound = ""), Eq(bound = ""), PartialEq(bound = ""), Debug(bound = ""))]
#[codec(encode_bound())]
#[codec(decode_bound())]
#[scale_info(bounds(), skip_type_params(Call))]
pub enum Xcm<Call> {
	/// Withdraw asset(s) (`assets`) from the ownership of `origin` and place them into `holding`. Execute the
	/// orders (`effects`).
	/// 从 "origin "的所有权中提取资产（"assets"）并将其放入 "holding"。执行订单（"effects"）。
	/// - `assets`: The asset(s) to be withdrawn into holding.
	/// - `effects`: The order(s) to execute on the holding account.
	/// - `assets': 资产被提现到持有者。
	/// - `effects`: 要在持有账户上执行的订单。
	/// Kind: *Instruction*.
	///
	/// Errors:
	#[codec(index = 0)]
	WithdrawAsset { assets: Vec<MultiAsset>, effects: Vec<Order<Call>> },

	/// Asset(s) (`assets`) have been received into the ownership of this system on the `origin` system.
	/// 资产（`assets`）已在`origin`系统上被接收到该系统的所有权中。
	/// Some orders are given (`effects`) which should be executed once the corresponding derivative assets have
	/// been placed into `holding`.
	/// 给出了一些订单（“effects”），一旦相应的衍生资产被“holding”，就应该执行这些订单。
	/// - `assets`: The asset(s) that are minted into holding.
	/// - `effects`: The order(s) to execute on the holding account.
	/// - `assets': 资产被铸造到持有者。
	/// - `effects`: 要在持有账户上执行的订单。
	/// Safety: `origin` must be trusted to have received and be storing `assets` such that they may later be
	/// withdrawn should this system send a corresponding message.
	/// 安全性："origin "必须被信任为已经收到并储存了 "assets"，如果这个系统发送了相应的信息，这些资产以后可以被撤回。
	/// Kind: *Trusted Indication*.
	///
	/// Errors:
	#[codec(index = 1)]
	ReserveAssetDeposit { assets: Vec<MultiAsset>, effects: Vec<Order<Call>> },

	/// Asset(s) (`assets`) have been destroyed on the `origin` system and equivalent assets should be
	/// created on this system.
	/// 资产（“assets”）已在“origin”系统上销毁，应在当前系统上创建等效资产。
	/// Some orders are given (`effects`) which should be executed once the corresponding derivative assets have
	/// been placed into `holding`.
	/// 给出了一些订单（“effects”），一旦相应的衍生资产被“holding”，就应该执行这些订单。
	/// - `assets`: The asset(s) that are minted into holding.
	/// - `effects`: The order(s) to execute on the holding account.
	/// - `assets': 资产被铸造到持有者。
	/// - `effects`: 要在持有账户上执行的订单。
	/// Safety: `origin` must be trusted to have irrevocably destroyed the `assets` prior as a consequence of
	/// sending this message.
	/// 安全性：必须相信“origin”在发送此消息之前已经不可撤销地销毁了“assets”。
	/// Kind: *Trusted Indication*.
	///
	/// Errors:
	#[codec(index = 2)]
	TeleportAsset { assets: Vec<MultiAsset>, effects: Vec<Order<Call>> },

	/// Indication of the contents of the holding account corresponding to the `QueryHolding` order of `query_id`.
	/// 指示对应于`query_id`的`QueryHolding`顺序的持有账户的内容。
	/// - `query_id`: The identifier of the query that resulted in this message being sent.
	/// - `assets`: The message content.
	/// - `query_id`: 发送此消息的查询的标识符。
	// 	- `assets`: 消息内容
	/// Safety: No concerns.
	///
	/// Kind: *Information*.
	///
	/// Errors:
	#[codec(index = 3)]
	QueryResponse {
		#[codec(compact)]
		query_id: u64,
		response: Response,
	},

	/// Withdraw asset(s) (`assets`) from the ownership of `origin` and place equivalent assets under the
	/// ownership of `dest` within this consensus system.
	/// 从 `origin` 的所有权中提取资产（`assets`），并将等效资产置于此共识系统中的 `dest` 所有权下。
	/// - `assets`: The asset(s) to be withdrawn.
	/// - `dest`: The new owner for the assets.
	/// - `assets`: 被提现的资产.
	// 	- `dest`: 资产的新拥有者.
	/// Safety: No concerns.
	///
	/// Kind: *Instruction*.
	///
	/// Errors:
	#[codec(index = 4)]
	TransferAsset { assets: Vec<MultiAsset>, dest: MultiLocation },

	/// Withdraw asset(s) (`assets`) from the ownership of `origin` and place equivalent assets under the
	/// ownership of `dest` within this consensus system.
	/// 从 `origin` 的所有权中提取资产（`assets`），并将等效资产置于此共识系统中的 `dest` 所有权下。
	/// Send an onward XCM message to `dest` of `ReserveAssetDeposit` with the given `effects`.
	/// 使用给定的 `effects` 向`ReserveAssetDeposit` 的`dest` 发送转发 XCM 消息。
	/// - `assets`: The asset(s) to be withdrawn.
	/// - `dest`: The new owner for the assets.
	/// - `effects`: The orders that should be contained in the `ReserveAssetDeposit` which is sent onwards to
	///   `dest`.
	/// - `assets`:想要提现的资产
	/// - `dest`: 资产的新拥有者
	/// - `effects`: 应包含在 `ReserveAssetDeposit` 中的订单将继续发送到 `dest`。
	/// Safety: No concerns.
	///
	/// Kind: *Instruction*.
	///
	/// Errors:
	#[codec(index = 5)]
	TransferReserveAsset { assets: Vec<MultiAsset>, dest: MultiLocation, effects: Vec<Order<()>> },

	/// Apply the encoded transaction `call`, whose dispatch-origin should be `origin` as expressed by the kind
	/// of origin `origin_type`.
	/// 应用编码的事务`call`，它的dispatch-origin应该是`origin`，由源类型`origin_type`表示。
	/// - `origin_type`: The means of expressing the message origin as a dispatch origin.
	/// - `max_weight`: The weight of `call`; this should be at least the chain's calculated weight and will
	///   be used in the weight determination arithmetic.
	/// - `call`: The encoded transaction to be applied.
	/// - `origin_type`: 表达消息来源，作为一个调度原点。
	/// - `max_weight`: `call`的权重；这应该至少是链的计算权重，并将用于权重确定的算术中。
	/// - `call': 要应用的编码交易。
	/// Safety: No concerns.
	///
	/// Kind: *Instruction*.
	///
	/// Errors:
	#[codec(index = 6)]
	Transact { origin_type: OriginKind, require_weight_at_most: u64, call: DoubleEncoded<Call> },

	/// A message to notify about a new incoming HRMP channel. This message is meant to be sent by the
	/// relay-chain to a para.
	/// 通知新传入 HRMP 通道的消息。此消息旨在由中继链发送到 平行链。
	/// - `sender`: The sender in the to-be opened channel. Also, the initiator of the channel opening.
	/// - `max_message_size`: The maximum size of a message proposed by the sender.
	/// - `max_capacity`: The maximum number of messages that can be queued in the channel.
	/// - `sender`: 将要打开的通道中的发送方。同时，也是打开通道的发起人。
	/// - `max_message_size`: 发送方提议的信息的最大尺寸。
	/// - `max_capacity': 信道中可排队的最大消息数。
	/// Safety: The message should originate directly from the relay-chain.
	/// 安全性：消息应该直接来自中继链。
	/// Kind: *System Notification*
	#[codec(index = 7)]
	HrmpNewChannelOpenRequest {
		#[codec(compact)]
		sender: u32,
		#[codec(compact)]
		max_message_size: u32,
		#[codec(compact)]
		max_capacity: u32,
	},

	/// A message to notify about that a previously sent open channel request has been accepted by
	/// the recipient. That means that the channel will be opened during the next relay-chain session
	/// change. This message is meant to be sent by the relay-chain to a para.
	/// 用于通知先前发送的开放频道请求已被收件人接受的消息。这意味着通道将在下一次中继链会话更改期间打开。
	/// 此消息旨在由中继链发送到平行链。
	/// Safety: The message should originate directly from the relay-chain.
	/// 安全性：消息应该直接来自中继链。
	/// Kind: *System Notification*
	///
	/// Errors:
	#[codec(index = 8)]
	HrmpChannelAccepted {
		#[codec(compact)]
		recipient: u32,
	},

	/// A message to notify that the other party in an open channel decided to close it. In particular,
	/// `initiator` is going to close the channel opened from `sender` to the `recipient`. The close
	/// will be enacted at the next relay-chain session change. This message is meant to be sent by
	/// the relay-chain to a para.
	/// 一条消息，通知对方在一个开放的频道决定关闭它。特别是，`initiator` 将关闭从`sender` 到`recipient` 的通道。
	/// 关闭将在下一次中继链会话更改时进行。此消息旨在由中继链发送到 平行链。
	/// Safety: The message should originate directly from the relay-chain.
	/// 安全性：消息应该直接来自中继链。
	/// Kind: *System Notification*
	///
	/// Errors:
	#[codec(index = 9)]
	HrmpChannelClosing {
		#[codec(compact)]
		initiator: u32,
		#[codec(compact)]
		sender: u32,
		#[codec(compact)]
		recipient: u32,
	},

	/// A message to indicate that the embedded XCM is actually arriving on behalf of some consensus
	/// location within the origin.
	///  表示嵌入的XCM实际上是代表原点内的某个共识位置而到达的信息。
	/// Safety: `who` must be an interior location of the context. This basically means that no `Parent`
	/// junctions are allowed in it. This should be verified at the time of XCM execution.
	/// 安全性：`who`必须是上下文的内部位置。这基本上意味着其中不允许有`Parent`结点。这应该在XCM执行时进行验证。
	/// Kind: *Instruction*
	///
	/// Errors:
	#[codec(index = 10)]
	RelayedFrom { who: MultiLocation, message: alloc::boxed::Box<Xcm<Call>> },
}

impl<Call> Xcm<Call> {
	pub fn into<C>(self) -> Xcm<C> {
		Xcm::from(self)
	}
	pub fn from<C>(xcm: Xcm<C>) -> Self {
		use Xcm::*;
		match xcm {
			WithdrawAsset { assets, effects } =>
				WithdrawAsset { assets, effects: effects.into_iter().map(Order::into).collect() },
			ReserveAssetDeposit { assets, effects } => ReserveAssetDeposit {
				assets,
				effects: effects.into_iter().map(Order::into).collect(),
			},
			TeleportAsset { assets, effects } =>
				TeleportAsset { assets, effects: effects.into_iter().map(Order::into).collect() },
			QueryResponse { query_id, response } => QueryResponse { query_id, response },
			TransferAsset { assets, dest } => TransferAsset { assets, dest },
			TransferReserveAsset { assets, dest, effects } =>
				TransferReserveAsset { assets, dest, effects },
			HrmpNewChannelOpenRequest { sender, max_message_size, max_capacity } =>
				HrmpNewChannelOpenRequest { sender, max_message_size, max_capacity },
			HrmpChannelAccepted { recipient } => HrmpChannelAccepted { recipient },
			HrmpChannelClosing { initiator, sender, recipient } =>
				HrmpChannelClosing { initiator, sender, recipient },
			Transact { origin_type, require_weight_at_most, call } =>
				Transact { origin_type, require_weight_at_most, call: call.into() },
			RelayedFrom { who, message } =>
				RelayedFrom { who, message: alloc::boxed::Box::new((*message).into()) },
		}
	}
}

pub mod opaque {
	/// The basic concrete type of `generic::Xcm`, which doesn't make any assumptions about the format of a
	/// call other than it is pre-encoded.
	/// `generic::Xcm` 的基本具体类型，除了预先编码之外，它不对调用的格式做任何假设。
	pub type Xcm = super::Xcm<()>;

	pub use super::order::opaque::*;
}

// Convert from a v1 response to a v0 response
// 转换v1应答为v0应答
impl TryFrom<Response1> for Response {
	type Error = ();
	fn try_from(new_response: Response1) -> result::Result<Self, ()> {
		Ok(match new_response {
			Response1::Assets(assets) => Self::Assets(assets.try_into()?),
			Response1::Version(..) => return Err(()),
		})
	}
}

impl<Call> TryFrom<Xcm1<Call>> for Xcm<Call> {
	type Error = ();
	fn try_from(x: Xcm1<Call>) -> result::Result<Xcm<Call>, ()> {
		use Xcm::*;
		Ok(match x {
			Xcm1::WithdrawAsset { assets, effects } => WithdrawAsset {
				assets: assets.try_into()?,
				effects: effects
					.into_iter()
					.map(Order::try_from)
					.collect::<result::Result<_, _>>()?,
			},
			Xcm1::ReserveAssetDeposited { assets, effects } => ReserveAssetDeposit {
				assets: assets.try_into()?,
				effects: effects
					.into_iter()
					.map(Order::try_from)
					.collect::<result::Result<_, _>>()?,
			},
			Xcm1::ReceiveTeleportedAsset { assets, effects } => TeleportAsset {
				assets: assets.try_into()?,
				effects: effects
					.into_iter()
					.map(Order::try_from)
					.collect::<result::Result<_, _>>()?,
			},
			Xcm1::QueryResponse { query_id, response } =>
				QueryResponse { query_id, response: response.try_into()? },
			Xcm1::TransferAsset { assets, beneficiary } =>
				TransferAsset { assets: assets.try_into()?, dest: beneficiary.try_into()? },
			Xcm1::TransferReserveAsset { assets, dest, effects } => TransferReserveAsset {
				assets: assets.try_into()?,
				dest: dest.try_into()?,
				effects: effects
					.into_iter()
					.map(Order::try_from)
					.collect::<result::Result<_, _>>()?,
			},
			Xcm1::HrmpNewChannelOpenRequest { sender, max_message_size, max_capacity } =>
				HrmpNewChannelOpenRequest { sender, max_message_size, max_capacity },
			Xcm1::HrmpChannelAccepted { recipient } => HrmpChannelAccepted { recipient },
			Xcm1::HrmpChannelClosing { initiator, sender, recipient } =>
				HrmpChannelClosing { initiator, sender, recipient },
			Xcm1::Transact { origin_type, require_weight_at_most, call } =>
				Transact { origin_type, require_weight_at_most, call: call.into() },
			Xcm1::RelayedFrom { who, message } => RelayedFrom {
				who: MultiLocation1 { interior: who, parents: 0 }.try_into()?,
				message: alloc::boxed::Box::new((*message).try_into()?),
			},
			Xcm1::SubscribeVersion { .. } | Xcm1::UnsubscribeVersion => return Err(()),
		})
	}
}
