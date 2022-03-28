// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Cross-Consensus Message format asset data structures.
//!
//! This encompasses four types for representing assets:
//! - `MultiAsset`: A description of a single asset, either an instance of a non-fungible or some amount of a fungible.
//! - `MultiAssets`: A collection of `MultiAsset`s. These are stored in a `Vec` and sorted with fungibles first.
//! - `Wild`: A single asset wildcard, this can either be "all" assets, or all assets of a specific kind.
//! - `MultiAssetFilter`: A combination of `Wild` and `MultiAssets` designed for efficiently filtering an XCM holding
//!   account.
//! 这包括了四种代表资产的类型。
//！ - `MultiAsset：一个单一资产的描述，可以是一个非可替换资产的实例，也可以是一些可替换资产的数量。
//！ - `MultiAssets`：一个`MultiAsset'的集合。这些资产被存储在一个`Vec'中，并以可替换资产为先进行排序。
//！ - `Wild`: 一个单一的资产通配符，它可以是 "all "资产，也可以是某一特定种类的所有资产。
//！ - `MultiAssetFilter`: Wild "和 "MultiAssets "的组合，用于有效过滤XCM持有账户。

use super::MultiLocation;
use alloc::{vec, vec::Vec};
use core::{
	cmp::Ordering,
	convert::{TryFrom, TryInto},
	result,
};
use parity_scale_codec::{self as codec, Decode, Encode};
use scale_info::TypeInfo;

/// A general identifier for an instance of a non-fungible asset class.
/// 不可替代资产类别实例的通用标识符。
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, TypeInfo)]
pub enum AssetInstance {
	/// Undefined - used if the non-fungible asset class has only one instance.
	Undefined,

	/// A compact index. Technically this could be greater than `u128`, but this implementation supports only
	/// values up to `2**128 - 1`.
	/// 一个紧凑的索引。从技术上讲，这可能大于 `u128`，但此实现仅支持高达 `2128 - 1` 的值。
	Index(#[codec(compact)] u128),

	/// A 4-byte fixed-length datum.
	Array4([u8; 4]),

	/// An 8-byte fixed-length datum.
	Array8([u8; 8]),

	/// A 16-byte fixed-length datum.
	Array16([u8; 16]),

	/// A 32-byte fixed-length datum.
	Array32([u8; 32]),

	/// An arbitrary piece of data. Use only when necessary.
	Blob(Vec<u8>),
}

impl From<()> for AssetInstance {
	fn from(_: ()) -> Self {
		Self::Undefined
	}
}

impl From<[u8; 4]> for AssetInstance {
	fn from(x: [u8; 4]) -> Self {
		Self::Array4(x)
	}
}

impl From<[u8; 8]> for AssetInstance {
	fn from(x: [u8; 8]) -> Self {
		Self::Array8(x)
	}
}

impl From<[u8; 16]> for AssetInstance {
	fn from(x: [u8; 16]) -> Self {
		Self::Array16(x)
	}
}

impl From<[u8; 32]> for AssetInstance {
	fn from(x: [u8; 32]) -> Self {
		Self::Array32(x)
	}
}

impl From<Vec<u8>> for AssetInstance {
	fn from(x: Vec<u8>) -> Self {
		Self::Blob(x)
	}
}

/// Classification of an asset being concrete or abstract.
/// 资产的分类是具体的还是抽象的。
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, Decode, TypeInfo)]
pub enum AssetId {
	Concrete(MultiLocation),
	Abstract(Vec<u8>),
}

impl<T: Into<MultiLocation>> From<T> for AssetId {
	fn from(x: T) -> Self {
		Self::Concrete(x.into())
	}
}

impl From<Vec<u8>> for AssetId {
	fn from(x: Vec<u8>) -> Self {
		Self::Abstract(x)
	}
}

impl AssetId {
	/// Prepend a `MultiLocation` to a concrete asset, giving it a new root location.
	/// 将“MultiLocation”添加到具体资产，为其提供新的根位置。
	pub fn prepend_with(&mut self, prepend: &MultiLocation) -> Result<(), ()> {
		if let AssetId::Concrete(ref mut l) = self {
			l.prepend_with(prepend.clone()).map_err(|_| ())?;
		}
		Ok(())
	}

	/// Mutate the asset to represent the same value from the perspective of a new `target`
	/// location. The local chain's location is provided in `ancestry`.
	/// 从新的“目标”位置的角度改变资产以表示相同的值。本地链的位置在“祖先”中提供。
	pub fn reanchor(&mut self, target: &MultiLocation, ancestry: &MultiLocation) -> Result<(), ()> {
		if let AssetId::Concrete(ref mut l) = self {
			l.reanchor(target, ancestry)?;
		}
		Ok(())
	}

	/// Use the value of `self` along with a `fun` fungibility specifier to create the corresponding `MultiAsset` value.
	/// 使用 `self` 的值和 `fun` 可替换性说明符来创建相应的 `MultiAsset` 值。
	pub fn into_multiasset(self, fun: Fungibility) -> MultiAsset {
		MultiAsset { fun, id: self }
	}

	/// Use the value of `self` along with a `fun` fungibility specifier to create the corresponding `WildMultiAsset`
	/// wildcard (`AllOf`) value.
	/// 使用 `self` 的值和 `fun` 可替代性说明符来创建相应的 `WildMultiAsset` 通配符 (`AllOf`) 值。
	pub fn into_wild(self, fun: WildFungibility) -> WildMultiAsset {
		WildMultiAsset::AllOf { fun, id: self }
	}
}

/// Classification of whether an asset is fungible or not, along with a mandatory amount or instance.
/// 资产是否可替代的分类，以及强制金额或实例。
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, Decode, TypeInfo)]
pub enum Fungibility {
	Fungible(#[codec(compact)] u128),
	NonFungible(AssetInstance),
}

impl Fungibility {
	pub fn is_kind(&self, w: WildFungibility) -> bool {
		use Fungibility::*;
		use WildFungibility::{Fungible as WildFungible, NonFungible as WildNonFungible};
		matches!((self, w), (Fungible(_), WildFungible) | (NonFungible(_), WildNonFungible))
	}
}

impl From<u128> for Fungibility {
	fn from(amount: u128) -> Fungibility {
		debug_assert_ne!(amount, 0);
		Fungibility::Fungible(amount)
	}
}

impl<T: Into<AssetInstance>> From<T> for Fungibility {
	fn from(instance: T) -> Fungibility {
		Fungibility::NonFungible(instance.into())
	}
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, TypeInfo)]
pub struct MultiAsset {
	pub id: AssetId,
	pub fun: Fungibility,
}

impl PartialOrd for MultiAsset {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for MultiAsset {
	fn cmp(&self, other: &Self) -> Ordering {
		match (&self.fun, &other.fun) {
			(Fungibility::Fungible(..), Fungibility::NonFungible(..)) => Ordering::Less,
			(Fungibility::NonFungible(..), Fungibility::Fungible(..)) => Ordering::Greater,
			_ => (&self.id, &self.fun).cmp(&(&other.id, &other.fun)),
		}
	}
}

impl<A: Into<AssetId>, B: Into<Fungibility>> From<(A, B)> for MultiAsset {
	fn from((id, fun): (A, B)) -> MultiAsset {
		MultiAsset { fun: fun.into(), id: id.into() }
	}
}

impl MultiAsset {
	pub fn is_fungible(&self, maybe_id: Option<AssetId>) -> bool {
		use Fungibility::*;
		matches!(self.fun, Fungible(..)) && maybe_id.map_or(true, |i| i == self.id)
	}

	pub fn is_non_fungible(&self, maybe_id: Option<AssetId>) -> bool {
		use Fungibility::*;
		matches!(self.fun, NonFungible(..)) && maybe_id.map_or(true, |i| i == self.id)
	}

	/// Prepend a `MultiLocation` to a concrete asset, giving it a new root location.
	pub fn prepend_with(&mut self, prepend: &MultiLocation) -> Result<(), ()> {
		self.id.prepend_with(prepend)
	}

	/// Mutate the location of the asset identifier if concrete, giving it the same location
	/// relative to a `target` context. The local context is provided as `ancestry`.
	pub fn reanchor(&mut self, target: &MultiLocation, ancestry: &MultiLocation) -> Result<(), ()> {
		self.id.reanchor(target, ancestry)
	}

	/// Mutate the location of the asset identifier if concrete, giving it the same location
	/// relative to a `target` context. The local context is provided as `ancestry`.
	pub fn reanchored(
		mut self,
		target: &MultiLocation,
		ancestry: &MultiLocation,
	) -> Result<Self, ()> {
		self.id.reanchor(target, ancestry)?;
		Ok(self)
	}

	/// Returns true if `self` is a super-set of the given `inner`.
	pub fn contains(&self, inner: &MultiAsset) -> bool {
		use Fungibility::*;
		if self.id == inner.id {
			match (&self.fun, &inner.fun) {
				(Fungible(a), Fungible(i)) if a >= i => return true,
				(NonFungible(a), NonFungible(i)) if a == i => return true,
				_ => (),
			}
		}
		false
	}
}

impl TryFrom<super::super::v0::MultiAsset> for MultiAsset {
	type Error = ();
	fn try_from(old: super::super::v0::MultiAsset) -> result::Result<MultiAsset, ()> {
		use super::super::v0::MultiAsset as V0;
		use AssetId::*;
		use Fungibility::*;
		let (id, fun) = match old {
			V0::ConcreteFungible { id, amount } => (Concrete(id.try_into()?), Fungible(amount)),
			V0::ConcreteNonFungible { class, instance } =>
				(Concrete(class.try_into()?), NonFungible(instance)),
			V0::AbstractFungible { id, amount } => (Abstract(id), Fungible(amount)),
			V0::AbstractNonFungible { class, instance } => (Abstract(class), NonFungible(instance)),
			_ => return Err(()),
		};
		Ok(MultiAsset { id, fun })
	}
}

impl TryFrom<super::super::v0::MultiAsset> for Option<MultiAsset> {
	type Error = ();
	fn try_from(old: super::super::v0::MultiAsset) -> result::Result<Option<MultiAsset>, ()> {
		match old {
			super::super::v0::MultiAsset::None => return Ok(None),
			x => return Ok(Some(x.try_into()?)),
		}
	}
}

impl TryFrom<Vec<super::super::v0::MultiAsset>> for MultiAsset {
	type Error = ();
	fn try_from(mut old: Vec<super::super::v0::MultiAsset>) -> result::Result<MultiAsset, ()> {
		if old.len() == 1 {
			old.remove(0).try_into()
		} else {
			Err(())
		}
	}
}

/// A `Vec` of `MultiAsset`s. There may be no duplicate fungible items in here and when decoding, they must be sorted.
/// `MultiAsset`s 的`Vec`。这里可能没有重复的可替代项目，解码时必须对其进行排序。
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, TypeInfo)]
pub struct MultiAssets(Vec<MultiAsset>);

impl Decode for MultiAssets {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
		Self::from_sorted_and_deduplicated(Vec::<MultiAsset>::decode(input)?)
			.map_err(|()| "Out of order".into())
	}
}

impl TryFrom<Vec<super::super::v0::MultiAsset>> for MultiAssets {
	type Error = ();
	fn try_from(old: Vec<super::super::v0::MultiAsset>) -> result::Result<MultiAssets, ()> {
		let v = old
			.into_iter()
			.map(Option::<MultiAsset>::try_from)
			.filter_map(|x| x.transpose())
			.collect::<result::Result<Vec<MultiAsset>, ()>>()?;
		Ok(v.into())
	}
}

impl From<Vec<MultiAsset>> for MultiAssets {
	fn from(mut assets: Vec<MultiAsset>) -> Self {
		let mut res = Vec::with_capacity(assets.len());
		if !assets.is_empty() {
			assets.sort();
			let mut iter = assets.into_iter();
			if let Some(first) = iter.next() {
				let last = iter.fold(first, |a, b| -> MultiAsset {
					match (a, b) {
						(
							MultiAsset { fun: Fungibility::Fungible(a_amount), id: a_id },
							MultiAsset { fun: Fungibility::Fungible(b_amount), id: b_id },
						) if a_id == b_id => MultiAsset {
							id: a_id,
							fun: Fungibility::Fungible(a_amount.saturating_add(b_amount)),
						},
						(
							MultiAsset { fun: Fungibility::NonFungible(a_instance), id: a_id },
							MultiAsset { fun: Fungibility::NonFungible(b_instance), id: b_id },
						) if a_id == b_id && a_instance == b_instance =>
							MultiAsset { fun: Fungibility::NonFungible(a_instance), id: a_id },
						(to_push, to_remember) => {
							res.push(to_push);
							to_remember
						},
					}
				});
				res.push(last);
			}
		}
		Self(res)
	}
}

impl<T: Into<MultiAsset>> From<T> for MultiAssets {
	fn from(x: T) -> Self {
		Self(vec![x.into()])
	}
}

impl MultiAssets {
	/// A new (empty) value.
	pub fn new() -> Self {
		Self(Vec::new())
	}

	/// Create a new instance of `MultiAssets` from a `Vec<MultiAsset>` whose contents are sorted and
	/// which contain no duplicates.
	/// `MultiAsset`s 的`Vec`。这里可能没有重复的可替代项目，解码时必须对其进行排序。
	/// Returns `Ok` if the operation succeeds and `Err` if `r` is out of order or had duplicates. If you can't
	/// guarantee that `r` is sorted and deduplicated, then use `From::<Vec<MultiAsset>>::from` which is infallible.
	/// 如果操作成功，则返回 `Ok`，如果 `r` 无序或有重复，则返回 `Err`。如果您不能保证 `r` 已排序和去重，
	/// 则使用绝对可靠的 `From::<Vec<MultiAsset>>::from`。
	pub fn from_sorted_and_deduplicated(r: Vec<MultiAsset>) -> Result<Self, ()> {
		if r.is_empty() {
			return Ok(Self(Vec::new()))
		}
		r.iter().skip(1).try_fold(&r[0], |a, b| -> Result<&MultiAsset, ()> {
			if a.id < b.id || a < b && (a.is_non_fungible(None) || b.is_non_fungible(None)) {
				Ok(b)
			} else {
				Err(())
			}
		})?;
		Ok(Self(r))
	}

	/// Create a new instance of `MultiAssets` from a `Vec<MultiAsset>` whose contents are sorted and
	/// which contain no duplicates.
	///
	/// In release mode, this skips any checks to ensure that `r` is correct, making it a negligible-cost operation.
	/// Generally though you should avoid using it unless you have a strict proof that `r` is valid.
	#[cfg(test)]
	pub fn from_sorted_and_deduplicated_skip_checks(r: Vec<MultiAsset>) -> Self {
		Self::from_sorted_and_deduplicated(r).expect("Invalid input r is not sorted/deduped")
	}
	/// Create a new instance of `MultiAssets` from a `Vec<MultiAsset>` whose contents are sorted and
	/// which contain no duplicates.
	///
	/// In release mode, this skips any checks to ensure that `r` is correct, making it a negligible-cost operation.
	/// Generally though you should avoid using it unless you have a strict proof that `r` is valid.
	///
	/// In test mode, this checks anyway and panics on fail.
	#[cfg(not(test))]
	pub fn from_sorted_and_deduplicated_skip_checks(r: Vec<MultiAsset>) -> Self {
		Self(r)
	}

	/// Add some asset onto the list, saturating. This is quite a laborious operation since it maintains the ordering.
	pub fn push(&mut self, a: MultiAsset) {
		if let Fungibility::Fungible(ref amount) = a.fun {
			for asset in self.0.iter_mut().filter(|x| x.id == a.id) {
				if let Fungibility::Fungible(ref mut balance) = asset.fun {
					*balance = balance.saturating_add(*amount);
					return
				}
			}
		}
		self.0.push(a);
		self.0.sort();
	}

	/// Returns `true` if this definitely represents no asset.
	pub fn is_none(&self) -> bool {
		self.0.is_empty()
	}

	/// Returns true if `self` is a super-set of the given `inner`.
	pub fn contains(&self, inner: &MultiAsset) -> bool {
		self.0.iter().any(|i| i.contains(inner))
	}

	/// Consume `self` and return the inner vec.
	pub fn drain(self) -> Vec<MultiAsset> {
		self.0
	}

	/// Return a reference to the inner vec.
	pub fn inner(&self) -> &Vec<MultiAsset> {
		&self.0
	}

	/// Return the number of distinct asset instances contained.
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Prepend a `MultiLocation` to any concrete asset items, giving it a new root location.
	pub fn prepend_with(&mut self, prefix: &MultiLocation) -> Result<(), ()> {
		self.0.iter_mut().try_for_each(|i| i.prepend_with(prefix))
	}

	/// Prepend a `MultiLocation` to any concrete asset items, giving it a new root location.
	pub fn reanchor(&mut self, target: &MultiLocation, ancestry: &MultiLocation) -> Result<(), ()> {
		self.0.iter_mut().try_for_each(|i| i.reanchor(target, ancestry))
	}

	/// Return a reference to an item at a specific index or `None` if it doesn't exist.
	pub fn get(&self, index: usize) -> Option<&MultiAsset> {
		self.0.get(index)
	}
}
/// Classification of whether an asset is fungible or not.
/// 资产是否可替代的分类。
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, Decode, TypeInfo)]
pub enum WildFungibility {
	Fungible,
	NonFungible,
}

/// A wildcard representing a set of assets.
/// 表示一组资产的通配符。
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, Decode, TypeInfo)]
pub enum WildMultiAsset {
	/// All assets in the holding register, up to `usize` individual assets (different instances of non-fungibles could
	/// be separate assets).
	/// 持有登记册中的所有资产，最多“使用”单个资产（不可替代的不同实例可以是单独的资产）。
	All,
	/// All assets in the holding register of a given fungibility and ID. If operating on non-fungibles, then a limit
	/// is provided for the maximum amount of matching instances.
	/// 给定可替代性和 ID 的持有登记册中的所有资产。如果在不可替代的设备上运行，则为匹配实例的最大数量提供限制。
	AllOf { id: AssetId, fun: WildFungibility },
}

impl TryFrom<super::super::v0::MultiAsset> for WildMultiAsset {
	type Error = ();
	fn try_from(old: super::super::v0::MultiAsset) -> result::Result<WildMultiAsset, ()> {
		use super::super::v0::MultiAsset as V0;
		use AssetId::*;
		use WildFungibility::*;
		let (id, fun) = match old {
			V0::All => return Ok(WildMultiAsset::All),
			V0::AllConcreteFungible { id } => (Concrete(id.try_into()?), Fungible),
			V0::AllConcreteNonFungible { class } => (Concrete(class.try_into()?), NonFungible),
			V0::AllAbstractFungible { id } => (Abstract(id), Fungible),
			V0::AllAbstractNonFungible { class } => (Abstract(class), NonFungible),
			_ => return Err(()),
		};
		Ok(WildMultiAsset::AllOf { id, fun })
	}
}

impl TryFrom<Vec<super::super::v0::MultiAsset>> for WildMultiAsset {
	type Error = ();
	fn try_from(mut old: Vec<super::super::v0::MultiAsset>) -> result::Result<WildMultiAsset, ()> {
		if old.len() == 1 {
			old.remove(0).try_into()
		} else {
			Err(())
		}
	}
}

impl WildMultiAsset {
	/// Returns true if `self` is a super-set of the given `inner`.
	///
	/// Typically, any wildcard is never contained in anything else, and a wildcard can contain any other non-wildcard.
	/// For more details, see the implementation and tests.
	pub fn contains(&self, inner: &MultiAsset) -> bool {
		use WildMultiAsset::*;
		match self {
			AllOf { fun, id } => inner.fun.is_kind(*fun) && &inner.id == id,
			All => true,
		}
	}

	/// Prepend a `MultiLocation` to any concrete asset components, giving it a new root location.
	pub fn reanchor(&mut self, target: &MultiLocation, ancestry: &MultiLocation) -> Result<(), ()> {
		use WildMultiAsset::*;
		match self {
			AllOf { ref mut id, .. } => id.reanchor(target, ancestry).map_err(|_| ()),
			All => Ok(()),
		}
	}
}

impl<A: Into<AssetId>, B: Into<WildFungibility>> From<(A, B)> for WildMultiAsset {
	fn from((id, fun): (A, B)) -> WildMultiAsset {
		WildMultiAsset::AllOf { fun: fun.into(), id: id.into() }
	}
}

/// `MultiAsset` collection, either `MultiAssets` or a single wildcard.
/// `MultiAsset` 集合，`MultiAssets` 或单个通配符。
/// Note: Vectors of wildcards whose encoding is supported in XCM v0 are unsupported
/// in this implementation and will result in a decode error.
/// 注意：XCM v0 中支持编码的通配符向量在此实现中不受支持，将导致解码错误。
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, Decode, TypeInfo)]
pub enum MultiAssetFilter {
	Definite(MultiAssets),
	Wild(WildMultiAsset),
}

impl<T: Into<WildMultiAsset>> From<T> for MultiAssetFilter {
	fn from(x: T) -> Self {
		Self::Wild(x.into())
	}
}

impl From<MultiAsset> for MultiAssetFilter {
	fn from(x: MultiAsset) -> Self {
		Self::Definite(vec![x].into())
	}
}

impl From<Vec<MultiAsset>> for MultiAssetFilter {
	fn from(x: Vec<MultiAsset>) -> Self {
		Self::Definite(x.into())
	}
}

impl From<MultiAssets> for MultiAssetFilter {
	fn from(x: MultiAssets) -> Self {
		Self::Definite(x)
	}
}

impl MultiAssetFilter {
	/// Returns true if `self` is a super-set of the given `inner`.
	///
	/// Typically, any wildcard is never contained in anything else, and a wildcard can contain any other non-wildcard.
	/// For more details, see the implementation and tests.
	pub fn contains(&self, inner: &MultiAsset) -> bool {
		match self {
			MultiAssetFilter::Definite(ref assets) => assets.contains(inner),
			MultiAssetFilter::Wild(ref wild) => wild.contains(inner),
		}
	}

	/// Prepend a `MultiLocation` to any concrete asset components, giving it a new root location.
	pub fn reanchor(&mut self, target: &MultiLocation, ancestry: &MultiLocation) -> Result<(), ()> {
		match self {
			MultiAssetFilter::Definite(ref mut assets) => assets.reanchor(target, ancestry),
			MultiAssetFilter::Wild(ref mut wild) => wild.reanchor(target, ancestry),
		}
	}
}

impl TryFrom<Vec<super::super::v0::MultiAsset>> for MultiAssetFilter {
	type Error = ();
	fn try_from(
		mut old: Vec<super::super::v0::MultiAsset>,
	) -> result::Result<MultiAssetFilter, ()> {
		if old.len() == 1 && old[0].is_wildcard() {
			Ok(MultiAssetFilter::Wild(old.remove(0).try_into()?))
		} else {
			Ok(MultiAssetFilter::Definite(old.try_into()?))
		}
	}
}
