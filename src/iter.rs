use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::vec::IntoIter;

use crate::partition::ordered_interval_partitions::OrderedIntervalPartitions;
use crate::set::ordered_integer_set::ContiguousIntegerSet;
use crate::set::traits::Refineable;
use crate::traits::SubsetIndexable;

/// `K` is the key type
/// `M` is the map type that maps keys of type `K` to values
pub trait UnionZip<K, M> {
    fn union_zip<'a>(&'a self, other: &'a M) -> UnionZipped<'a, K, M>;
}

pub trait IntoUnionZip<'a, K, M> {
    fn into_union_zip(self, other: &'a M) -> UnionZipped<'a, K, M>;
}

pub struct UnionZipped<'a, K, M> {
    keys: Vec<K>,
    maps: Vec<&'a M>,
}

pub struct UnionZippedIter<'a, K, M, I: Iterator<Item=K>> {
    keys: I,
    maps: Vec<&'a M>,
}

/// Takes the sorted union of the two sets of keys for future iteration
/// ```
/// use std::collections::HashMap;
/// use analytic::iter::UnionZip;
/// let m1: HashMap<i32, i32> = vec![
///         (1, 10),
///         (3, 23),
///         (4, 20)
///     ].into_iter().collect();
/// let m2: HashMap<i32, i32> = vec![
///         (0, 4),
///         (1, 20),
///         (4, 20),
///         (9, 29),
///     ].into_iter().collect();
///
/// let mut iter = m1.union_zip(&m2)
///                  .into_iter();
/// assert_eq!(Some((0, vec![None, Some(&4)])), iter.next());
/// assert_eq!(Some((1, vec![Some(&10), Some(&20)])), iter.next());
/// assert_eq!(Some((3, vec![Some(&23), None])), iter.next());
/// assert_eq!(Some((4, vec![Some(&20), Some(&20)])), iter.next());
/// assert_eq!(Some((9, vec![None, Some(&29)])), iter.next());
/// assert_eq!(None, iter.next());
/// ```
impl<K, V> UnionZip<K, HashMap<K, V>> for HashMap<K, V>
    where K: Hash + Eq + Clone + Ord {
    fn union_zip<'a>(&'a self, other: &'a Self) -> UnionZipped<'a, K, HashMap<K, V>> {
        let mut keys: Vec<K> = self
            .keys()
            .collect::<HashSet<&K>>()
            .union(
                &other
                    .keys()
                    .collect::<HashSet<&K>>()
            )
            .map(|&k| k.clone())
            .collect();

        keys.sort();

        UnionZipped {
            keys,
            maps: vec![&self, other],
        }
    }
}

impl<'a, K, V> IntoUnionZip<'a, K, HashMap<K, V>> for UnionZipped<'a, K, HashMap<K, V>>
    where K: Hash + Eq + Clone + Ord {
    fn into_union_zip(self, other: &'a HashMap<K, V>) -> UnionZipped<'a, K, HashMap<K, V>> {
        let mut keys: Vec<K> =
            self.keys
                .iter()
                .collect::<HashSet<&K>>()
                .union(
                    &other
                        .keys()
                        .collect::<HashSet<&K>>()
                )
                .map(|&k| k.clone())
                .collect();

        keys.sort();

        let mut maps = self.maps.clone();
        maps.push(other);
        UnionZipped {
            keys,
            maps,
        }
    }
}

impl<'a, K, V> IntoIterator for UnionZipped<'a, K, HashMap<K, V>>
    where K: Hash + Eq {
    type Item = <UnionZippedIter<'a, K, HashMap<K, V>, IntoIter<K>> as Iterator>::Item;
    type IntoIter = UnionZippedIter<'a, K, HashMap<K, V>, IntoIter<K>>;
    fn into_iter(self) -> Self::IntoIter {
        UnionZippedIter {
            keys: self.keys.into_iter(),
            maps: self.maps,
        }
    }
}

impl<'a, K, V> Iterator for UnionZippedIter<'a, K, HashMap<K, V>, IntoIter<K>>
    where K: Hash + Eq {
    type Item = (K, Vec<Option<&'a V>>);
    fn next(&mut self) -> Option<Self::Item> {
        match self.keys.next() {
            None => None,
            Some(k) => {
                let mapped: Vec<Option<&'a V>> =
                    self.maps
                        .iter()
                        .map(
                            |m| if m.contains_key(&k) {
                                Some(&m[&k])
                            } else {
                                None
                            })
                        .collect();

                Some((k, mapped))
            }
        }
    }
}

pub trait CommonRefinementZip<S, K, M> {
    fn common_refinement_zip<'a, F>(&'a self, other: &'a Self, sort_fn: F)
        -> CommonRefinementZipped<'a, S, K, M>
        where S: SubsetIndexable<K>, F: FnMut(&K, &K) -> Ordering;
}

pub struct CommonRefinementZipped<'a, S, K, M>
    where S: SubsetIndexable<K> {
    refined_keys: Vec<K>,
    left_keys: S,
    right_keys: S,
    left: &'a M,
    right: &'a M,
}

pub struct CommonRefinementZipIter<'a, S, K, M, I: Iterator<Item=K>>
    where S: SubsetIndexable<K> {
    refined_keys_iter: I,
    left_keys: S,
    right_keys: S,
    left: &'a M,
    right: &'a M,
}

pub type UsizeInterval = ContiguousIntegerSet<usize>;

impl<V> CommonRefinementZip<
    OrderedIntervalPartitions<usize>,
    UsizeInterval,
    HashMap<UsizeInterval, V>
> for HashMap<UsizeInterval, V> {
    /// ```
    /// use std::collections::HashMap;
    /// use analytic::interval::traits::Interval;
    /// use analytic::iter::{CommonRefinementZip, UsizeInterval};
    ///
    /// let m1: HashMap<UsizeInterval, i32> = vec![
    ///     (UsizeInterval::new(0usize, 5), 5),
    ///     (UsizeInterval::new(8, 10), 2),
    /// ].into_iter().collect();
    ///
    /// let m2: HashMap<UsizeInterval, i32> = vec![
    ///     (UsizeInterval::new(2usize, 4), 8),
    ///     (UsizeInterval::new(12, 13), 9),
    /// ].into_iter().collect();
    ///
    /// let mut iter = m1
    ///     .common_refinement_zip(&m2, |a, b| a.get_start().cmp(&b.get_start()))
    ///     .into_iter();
    /// assert_eq!(Some((UsizeInterval::new(0usize, 1), (Some(&5), None))), iter.next());
    /// assert_eq!(Some((UsizeInterval::new(2usize, 4), (Some(&5), Some(&8)))), iter.next());
    /// assert_eq!(Some((UsizeInterval::new(5usize, 5), (Some(&5), None))), iter.next());
    /// assert_eq!(Some((UsizeInterval::new(8usize, 10), (Some(&2), None))), iter.next());
    /// assert_eq!(Some((UsizeInterval::new(12usize, 13), (None, Some(&9)))), iter.next());
    /// ```
    fn common_refinement_zip<'a, F>(&'a self, other: &'a Self, mut sort_fn: F)
        -> CommonRefinementZipped<'a, OrderedIntervalPartitions<usize>, UsizeInterval, HashMap<UsizeInterval, V>>
        where F: FnMut(&UsizeInterval, &UsizeInterval) -> Ordering {
        let mut keys_1: Vec<UsizeInterval> = self.keys().map(|i| *i).collect();
        keys_1.sort_by(|a, b| sort_fn(a, b));

        let mut keys_2: Vec<UsizeInterval> = other.keys().map(|i| *i).collect();
        keys_2.sort_by(|a, b| sort_fn(a, b));

        let left_keys = OrderedIntervalPartitions::from_vec_with_trusted_order(keys_1.clone());
        let right_keys = OrderedIntervalPartitions::from_vec_with_trusted_order(keys_2.clone());

        let refined_keys = left_keys
            .get_common_refinement(&right_keys)
            .into_vec();

        CommonRefinementZipped {
            refined_keys,
            left_keys,
            right_keys,
            left: &self,
            right: other,
        }
    }
}

impl<'a, V> IntoIterator for CommonRefinementZipped<
    'a,
    OrderedIntervalPartitions<usize>,
    UsizeInterval,
    HashMap<UsizeInterval, V>
> {
    type Item = <CommonRefinementZipIter<
        'a,
        OrderedIntervalPartitions<usize>,
        UsizeInterval,
        HashMap<UsizeInterval, V>,
        IntoIter<UsizeInterval>
    > as Iterator>::Item;

    type IntoIter = CommonRefinementZipIter<
        'a,
        OrderedIntervalPartitions<usize>,
        UsizeInterval,
        HashMap<UsizeInterval, V>,
        IntoIter<UsizeInterval>
    >;

    fn into_iter(self) -> Self::IntoIter {
        CommonRefinementZipIter {
            refined_keys_iter: self.refined_keys.into_iter(),
            left_keys: self.left_keys,
            right_keys: self.right_keys,
            left: self.left,
            right: self.right,
        }
    }
}

impl<'a, V> Iterator for CommonRefinementZipIter<
    'a,
    OrderedIntervalPartitions<usize>,
    UsizeInterval,
    HashMap<UsizeInterval, V>,
    IntoIter<UsizeInterval>
> {
    type Item = (UsizeInterval, (Option<&'a V>, Option<&'a V>));
    fn next(&mut self) -> Option<Self::Item> {
        match self.refined_keys_iter.next() {
            None => None,
            Some(k) => {
                let left = match self.left_keys.get_set_containing(&k) {
                    None => None,
                    Some(s) => Some(&self.left[&s])
                };
                let right = match self.right_keys.get_set_containing(&k) {
                    None => None,
                    Some(s) => Some(&self.right[&s])
                };
                Some((k, (left, right)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union_zip_iter_hashmap() {
        let m1: HashMap<i32, i32> = vec![
            // 0
            (1, 10),
            (3, 23),
            (4, 20),
            // 9
            (12, 6),
            // 14
        ].into_iter().collect();
        let m2: HashMap<i32, i32> = vec![
            (0, 4),
            (1, 20),
            // 3
            (4, 20),
            (9, 29),
            // 14
        ].into_iter().collect();

        let mut iter = m1.union_zip(&m2)
                         .into_iter();
        assert_eq!(Some((0, vec![None, Some(&4)])), iter.next());
        assert_eq!(Some((1, vec![Some(&10), Some(&20)])), iter.next());
        assert_eq!(Some((3, vec![Some(&23), None])), iter.next());
        assert_eq!(Some((4, vec![Some(&20), Some(&20)])), iter.next());
        assert_eq!(Some((9, vec![None, Some(&29)])), iter.next());
        assert_eq!(Some((12, vec![Some(&6), None])), iter.next());
        assert_eq!(None, iter.next());

        let m3: HashMap<i32, i32> = vec![
            (0, 9),
            // 1
            (3, 43),
            (4, 8),
            // 9
            // 12
            (14, 68),
        ].into_iter().collect();

        let mut iter2 = m1
            .union_zip(&m2)
            .into_union_zip(&m3)
            .into_iter();
        assert_eq!(Some((0, vec![None, Some(&4), Some(&9)])), iter2.next());
        assert_eq!(Some((1, vec![Some(&10), Some(&20), None])), iter2.next());
        assert_eq!(Some((3, vec![Some(&23), None, Some(&43)])), iter2.next());
        assert_eq!(Some((4, vec![Some(&20), Some(&20), Some(&8)])), iter2.next());
        assert_eq!(Some((9, vec![None, Some(&29), None])), iter2.next());
        assert_eq!(Some((12, vec![Some(&6), None, None])), iter2.next());
        assert_eq!(Some((14, vec![None, None, Some(&68)])), iter2.next());
        assert_eq!(None, iter2.next());

        let m4: HashMap<i32, i32> = vec![
            // 0
            // 1
            // 3
            (4, 73),
            // 9
            // 12
            (14, 64),
        ].into_iter().collect();

        let mut iter3 = m1
            .union_zip(&m2)
            .into_union_zip(&m3)
            .into_union_zip(&m4)
            .into_iter();
        assert_eq!(Some((0, vec![None, Some(&4), Some(&9), None])), iter3.next());
        assert_eq!(Some((1, vec![Some(&10), Some(&20), None, None])), iter3.next());
        assert_eq!(Some((3, vec![Some(&23), None, Some(&43), None])), iter3.next());
        assert_eq!(Some((4, vec![Some(&20), Some(&20), Some(&8), Some(&73)])), iter3.next());
        assert_eq!(Some((9, vec![None, Some(&29), None, None])), iter3.next());
        assert_eq!(Some((12, vec![Some(&6), None, None, None])), iter3.next());
        assert_eq!(Some((14, vec![None, None, Some(&68), Some(&64)])), iter3.next());
        assert_eq!(None, iter3.next());
    }
}
