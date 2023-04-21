# range_bounds_map

[![License](https://img.shields.io/github/license/ripytide/range_bounds_map)](https://www.gnu.org/licenses/agpl-3.0.en.html)
[![Docs](https://docs.rs/range_bounds_map/badge.svg)](https://docs.rs/range_bounds_map)
[![Maintained](https://img.shields.io/maintenance/yes/2023)](https://github.com/ripytide)
[![Crates.io](https://img.shields.io/crates/v/range_bounds_map)](https://crates.io/crates/range_bounds_map)

<p align="center">
<img src="logo.png" alt="range_bounds_map_logo" width="350">
</p>

This crate provides [`RangeBoundsMap`] and [`RangeBoundsSet`], Data
Structures for storing non-overlapping intervals based of [`BTreeMap`].

## Example using [`Range`]s

```rust
use range_bounds_map::test_ranges::ie;
use range_bounds_map::RangeBoundsMap;

let mut map = RangeBoundsMap::new();

map.insert_strict(ie(0, 5), true);
map.insert_strict(ie(5, 10), false);

assert_eq!(map.overlaps(ie(-2, 12)), true);
assert_eq!(map.contains_point(20), false);
assert_eq!(map.contains_point(5), true);
```

## Example using a custom [`RangeBounds`] type

```rust
use std::ops::{Bound, RangeBounds};

use range_bounds_map::test_ranges::ie;
use range_bounds_map::RangeBoundsMap;
use range_bounds_map::FiniteRange;

#[derive(Debug, Copy, Clone)]
enum Reservation {
	// Start, End (Inclusive-Exclusive)
	Finite(i8, i8),
	// Start (Inclusive-Forever)
	Infinite(i8),
}

// First, we need to implement FiniteRange
impl FiniteRange<i8> for Reservation {
    fn start(&self) -> i8 {
        match self {
            Reservation::Finite(start, _) => *start,
            Reservation::Infinite(start) => *start,
        }
    }
    fn end(&self) -> i8 {
        match self {
            //the end is exclusive so we take off 1 with checking
            //for compile time error overflow detection
            Reservation::Finite(_, end) => end.checked_sub(1).unwrap(),
            Reservation::Infinite(_) => i8::MAX,
        }
    }
}

// Next we can create a custom typed RangeBoundsMap
let reservation_map = RangeBoundsMap::from_slice_strict([
	(Reservation::Finite(10, 20), "Ferris".to_string()),
	(Reservation::Infinite(20), "Corro".to_string()),
])
.unwrap();

for (reservation, name) in reservation_map.overlapping(ie(16, 17))
{
	println!(
		"{name} has reserved {reservation:?} inside the range 16..17"
	);
}

for (reservation, name) in reservation_map.iter() {
	println!("{name} has reserved {reservation:?}");
}

assert_eq!(
	reservation_map.overlaps(Reservation::Infinite(0)),
	true
);
```

## Key Definitions:

### Invalid Ranges

Within this crate, not all ranges are considered valid
ranges. The definition of the validity of a range used
within this crate is that a range is only valid if it contains
at least one value of the underlying domain.

For example, `4..6` is considered valid as it contains the values
`4` and `5`, however, `4..4` is considered invalid as it contains
no values. Another example of invalid range are those whose start
values are greater than their end values. such as `5..2` or
`100..=40`.

Here are a few examples of ranges and whether they are valid:

| range          | valid |
| -------------- | ----- |
| 0..=0          | YES   |
| 0..0           | NO    |
| 0..1           | YES   |
| 9..8           | NO    |
| (0.4)..=(-0.2) | NO    |
| ..(-3)         | YES   |
| 0.0003..       | YES   |
| ..             | YES   |
| 400..=400      | YES   |

### Overlap

Two ranges are "overlapping" if there exists a point that is contained
within both ranges.

### Touching

Two ranges are "touching" if they do not overlap and there exists no
value between them. For example, `2..4` and `4..6` are touching but
`2..4` and `6..8` are not, neither are `2..6` and `4..8`.

### Merging

When a range "merges" other ranges it absorbs them to become larger.

### Further Reading

See Wikipedia's article on mathematical Intervals:
<https://en.wikipedia.org/wiki/Interval_(mathematics)>

# Credit

I originally came up with the `StartBound`: [`Ord`] bodge on my own,
however, I later stumbled across [`rangemap`] which also used a
`StartBound`: [`Ord`] bodge. [`rangemap`] then became my main source
of inspiration.

Later I then undid the [`Ord`] bodge and switched to my own full-code
port of [`BTreeMap`], inspired and forked from [`copse`], for it's
increased flexibility.

# Origin

The aim for this library was to become a more generic superset of
[`rangemap`], following from [this
issue](https://github.com/jeffparsons/rangemap/issues/56) and [this
pull request](https://github.com/jeffparsons/rangemap/pull/57) in
which I changed [`rangemap`]'s [`RangeMap`] to use [`RangeBounds`]s as
keys before I realized it might be easier and simpler to just write it
all from scratch.

# Similar Crates

Here are some relevant crates I found whilst searching around the
topic area:

- <https://docs.rs/rangemap>
  Very similar to this crate but can only use [`Range`]s and
  [`RangeInclusive`]s as keys in it's `map` and `set` structs (separately).
- <https://docs.rs/btree-range-map>
- <https://docs.rs/ranges>
  Cool library for fully-generic ranges (unlike std::ops ranges), along
  with a `Ranges` datastructure for storing them (Vec-based
  unfortunately)
- <https://docs.rs/intervaltree>
  Allows overlapping intervals but is immutable unfortunately
- <https://docs.rs/nonoverlapping_interval_tree>
  Very similar to rangemap except without a `gaps()` function and only
  for [`Range`]s and not [`RangeInclusive`]s. And also no fancy
  merging functions.
- <https://docs.rs/unbounded-interval-tree>
  A data structure based off of a 2007 published paper! It supports any
  RangeBounds as keys too, except it is implemented with a non-balancing
  `Box<Node>` based tree, however it also supports overlapping
  RangeBounds which my library does not.
- <https://docs.rs/rangetree>
  I'm not entirely sure what this library is or isn't, but it looks like
  a custom red-black tree/BTree implementation used specifically for a
  Range Tree. Interesting but also quite old (5 years) and uses
  unsafe.

[`btreemap`]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
[`btreeset`]: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html
[`rangebounds`]: https://doc.rust-lang.org/std/ops/trait.RangeBounds.html
[`start_bound()`]: https://doc.rust-lang.org/std/ops/trait.RangeBounds.html#tymethod.start_bound
[`end_bound()`]: https://doc.rust-lang.org/std/ops/trait.RangeBounds.html#tymethod.end_bound
[`range`]: https://doc.rust-lang.org/std/ops/struct.Range.html
[`range()`]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html#method.range
[`rangemap`]: https://docs.rs/rangemap/latest/rangemap/
[`rangeinclusivemap`]: https://docs.rs/rangemap/latest/rangemap/inclusive_map/struct.RangeInclusiveMap.html#
[`rangeinclusive`]: https://doc.rust-lang.org/std/ops/struct.RangeInclusive.html
[`ord`]: https://doc.rust-lang.org/std/cmp/trait.Ord.html
[`rangeboundsmap`]: https://docs.rs/range_bounds_map/latest/range_bounds_map/range_bounds_map/struct.RangeBoundsMap.html
[`rangeboundsset`]: https://docs.rs/range_bounds_map/latest/range_bounds_map/range_bounds_set/struct.RangeBoundsSet.html
[`copse`]: https://github.com/eggyal/copse
