/*
Copyright 2022 James Forster

This file is part of range_bounds_map.

range_bounds_map is free software: you can redistribute it and/or
modify it under the terms of the GNU General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

range_bounds_map is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
General Public License for more details.

You should have received a copy of the GNU General Public License
along with range_bounds_map. If not, see <https://www.gnu.org/licenses/>.
*/

//! This crate provides [`RangeBoundsMap`] and [`RangeBoundsSet`].
//!
//! [`RangeBoundsMap`] is similar to [`BTreeMap`] except [`RangeBoundsMap`]
//! uses any type that implements the [`RangeBounds`] trait as keys, while
//! maintaining two invariants:
//! - No two keys may overlap
//! - A keys' [`start_bound()`] <= its [`end_bound()`]
//!
//! [`RangeBoundsSet`] is like [`RangeBoundsMap`] except it
//! uses `()` as values, as [`BTreeSet`] does for [`BTreeMap`]
//!
//! # Example using [`Range`]s
//!
//! ```
//! use range_bounds_map::RangeBoundsMap;
//!
//! let mut range_bounds_map = RangeBoundsMap::new();
//!
//! range_bounds_map.insert(0..5, true);
//! range_bounds_map.insert(5..10, false);
//!
//! assert_eq!(range_bounds_map.overlaps(&(-2..12)), true);
//! assert_eq!(range_bounds_map.contains_point(&20), false);
//! assert_eq!(range_bounds_map.contains_point(&5), true);
//! ```
//!
//! # Example using a custom [`RangeBounds`] type
//! ```
//! use std::ops::{Bound, RangeBounds};
//!
//! use range_bounds_map::RangeBoundsMap;
//!
//! #[derive(Debug)]
//! enum Reservation {
//! 	// Start, End (Inclusive-Inclusive)
//! 	Finite(u8, u8),
//! 	// Start (Exclusive)
//! 	Infinite(u8),
//! }
//!
//! // First, we need to implement RangeBounds
//! impl RangeBounds<u8> for Reservation {
//! 	fn start_bound(&self) -> Bound<&u8> {
//! 		match self {
//! 			Reservation::Finite(start, _) => {
//! 				Bound::Included(start)
//! 			}
//! 			Reservation::Infinite(start) => {
//! 				Bound::Excluded(start)
//! 			}
//! 		}
//! 	}
//! 	fn end_bound(&self) -> Bound<&u8> {
//! 		match self {
//! 			Reservation::Finite(_, end) => Bound::Included(end),
//! 			Reservation::Infinite(_) => Bound::Unbounded,
//! 		}
//! 	}
//! }
//!
//! // Next we can create a custom typed RangeBoundsMap
//! let reservation_map = RangeBoundsMap::try_from([
//! 	(Reservation::Finite(10, 20), "Ferris".to_string()),
//! 	(Reservation::Infinite(20), "Corro".to_string()),
//! ])
//! .unwrap();
//!
//! for (reservation, name) in reservation_map.overlapping(&(16..17))
//! {
//! 	println!(
//! 		"{name} has reserved {reservation:?} inside the range 16..17"
//! 	);
//! }
//!
//! for (reservation, name) in reservation_map.iter() {
//! 	println!("{name} has reserved {reservation:?}");
//! }
//!
//! assert_eq!(
//! 	reservation_map.overlaps(&Reservation::Infinite(0)),
//! 	true
//! );
//! ```
//!
//! # How
//!
//! Most of the [`RangeBounds`]-specific methods on [`RangeBoundsMap`]
//! utilize the [`RangeBoundsMap::overlapping()`] method which
//! internally uses [`BTreeMap`]'s [`range()`] function. To allow
//! using [`range()`] for this purpose a newtype wrapper is wrapped
//! around the [`start_bound()`]s so that we can apply our custom [`Ord`]
//! implementation onto all the [`start_bound()`]s.
//!
//! # Improvements/Caveats
//!
//! There are a few issues I can think of with this implementation,
//! each of them are documented as GitHub Issues. If you would like
//! any of these features added, drop a comment in a respective GitHub
//! Issue (or even open a new one) and I'd be happy to implement it.
//!
//! To summarise:
//!
//! - No coalescing/merge insert functions, yet
//! - No `gaps()` iterator function, yet
//! - Missing some functions common to BTreeMap and BTreeSet like:
//!   - `clear()`
//!   - `is_subset()`
//!   - etc... a bunch more
//! - Sub-optimal use of unnecessary `cloned()` just to placate the borrow checker
//! - The data structures are lacking a lot of useful traits, such as:
//!   - Serde: Serialize and Deserialize
//!   - FromIterator
//!   - IntoIterator
//!   - Probably a bunch more
//!
//! # Credit
//!
//! I originally came up with the `StartBound`: [`Ord`] bodge on my
//! own, however, I later stumbled across [`rangemap`] which also used
//! a `StartBound`: [`Ord`] bodge. [`rangemap`] then became my main
//! source of inspiration. The aim for my library was to become a more
//! generic superset of [`rangemap`], following from
//! [this issue](https://github.com/jeffparsons/rangemap/issues/56) and
//! [this pull request](https://github.com/jeffparsons/rangemap/pull/57)
//! in which I changed [`rangemap`]'s [`RangeMap`] to use
//! [`RangeBounds`]s as keys before I realized it might be easier and
//! simpler to just write it all from scratch. Which ended up working
//! really well with some simplifications I made which ended up
//! resulting in much less code (~600 lines over `rangemap`'s ~2700)
//!
//! # Similar Crates
//!
//! Here are some relevant crates I found whilst searching around the
//! topic area:
//!
//! - <https://docs.rs/rangemap>
//!   Very similar to this crate but can only use [`Range`]s and
//!   [`RangeInclusive`]s as keys in it's `map` and `set` structs (separately).
//! - <https://docs.rs/ranges>
//!   Cool library for fully-generic ranges (unlike std::ops ranges), along
//!   with a `Ranges` datastructure for storing them (Vec-based
//!   unfortunately)
//! - <https://docs.rs/intervaltree>
//!   Allows overlapping intervals but is immutable unfortunately
//! - <https://docs.rs/nonoverlapping_interval_tree>
//!   Very similar to rangemap except without a `gaps()` function and only
//!   for [`Range`]s and not [`RangeInclusive`]s. And also no fancy coalescing
//!   functions.
//! - <https://docs.rs/unbounded-interval-tree>
//!   A data structure based off of a 2007 published paper! It supports any
//!   RangeBounds as keys too, except it is implemented with a non-balancing
//!   `Box<Node>` based tree, however it also supports overlapping
//!   RangeBounds which my library does not.
//! - <https://docs.rs/rangetree>
//!   I'm not entirely sure what this library is or isn't, but it looks like
//!   a custom red-black tree/BTree implementation used specifically for a
//!   Range Tree. Interesting but also quite old (5 years) and uses
//!   unsafe.
//!
//!
//! [`BTreeMap`]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
//! [`BTreeSet`]: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html
//! [`RangeBounds`]: https://doc.rust-lang.org/std/ops/trait.RangeBounds.html
//! [`start_bound()`]: https://doc.rust-lang.org/std/ops/trait.RangeBounds.html#tymethod.start_bound
//! [`end_bound()`]: https://doc.rust-lang.org/std/ops/trait.RangeBounds.html#tymethod.end_bound
//! [`Range`]: https://doc.rust-lang.org/std/ops/struct.Range.html
//! [`range()`]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html#method.range
//! [`rangemap`]: https://docs.rs/rangemap/latest/rangemap/
//! [`RangeInclusiveMap`]: https://docs.rs/rangemap/latest/rangemap/inclusive_map/struct.RangeInclusiveMap.html#
//! [`RangeInclusive`]: https://doc.rust-lang.org/std/ops/struct.RangeInclusive.html

#![feature(is_some_and)]
#![feature(let_chains)]
#![allow(clippy::tabs_in_doc_comments)]
#![allow(clippy::needless_return)]
pub(crate) mod bounds;
pub mod range_bounds_map;
pub mod range_bounds_set;

pub use crate::range_bounds_map::{InsertError, RangeBoundsMap};
pub use crate::range_bounds_set::RangeBoundsSet;
