use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Debug;

/// A Iterator that run merge sort on `N` ordered iterators.
///
/// This implementation is optimized based on the assumption that most iterators are empty.
/// Compare to first implementation at [rust playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=6919618250c88342b73e5faeb513d212)
/// ```not rust
/// Benchmarking test_merge_sort_hashmap
/// Benchmarking test_merge_sort_hashmap: Warming up for 3.0000 s
/// Benchmarking test_merge_sort_hashmap: Collecting 100 samples in estimated 5.2910 s (86k iterations)
/// Benchmarking test_merge_sort_hashmap: Analyzing
/// test_merge_sort_hashmap time:   [54.360 us 54.504 us 54.703 us]
/// Found 12 outliers among 100 measurements (12.00%)
///   3 (3.00%) high mild
///   9 (9.00%) high severe
///
/// Benchmarking test_merge_sort_origin
/// Benchmarking test_merge_sort_origin: Warming up for 3.0000 s
/// Benchmarking test_merge_sort_origin: Collecting 100 samples in estimated 5.8930 s (10k iterations)
/// Benchmarking test_merge_sort_origin: Analyzing
/// test_merge_sort_origin  time:   [582.36 us 585.10 us 588.14 us]
/// Found 10 outliers among 100 measurements (10.00%)
///   8 (8.00%) high mild
///   2 (2.00%) high severe
/// ```
#[derive(Debug)]
pub struct MergeSortIterator<T, I> {
    sources: HashMap<usize, I>,
    buffered: HashMap<usize, T>,
    #[cfg(debug_assertions)]
    last_elements: HashMap<usize, T>,
    ordering: Order,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Order {
    Asc,
    Desc,
}

impl<T: Clone + Debug + Ord, I: Iterator<Item = T>> MergeSortIterator<T, I> {
    /// Build a `MergeSortIterator` from a array of Iterator
    pub fn new(sources: Vec<I>, ordering: Order) -> Self {
        let (buffered, sources): (HashMap<usize, T>, HashMap<usize, I>) = sources
            .into_iter()
            .map(|mut iter| (iter.next(), iter))
            .filter(|(next, _iter)| next.is_some())
            .enumerate()
            .map(|(idx, (next, iter))| ((idx, next.unwrap()), (idx, iter)))
            .unzip();
        let size = sources.len();
        Self {
            sources,
            buffered,
            #[cfg(debug_assertions)]
            last_elements: HashMap::with_capacity(size),
            ordering,
        }
    }

    /// Find the most `ordering` element in the `buffered` array.
    /// ## panics
    /// When the elements in `buffered` are all `None`, calling to this function will panics.
    fn arg_cmp(&mut self) -> usize {
        use Order::*;

        let tmp = self.buffered.iter();
        *match self.ordering {
            Asc => tmp.min_by(|(_idx, x), (_idy, y)| x.cmp(y)),
            Desc => tmp.max_by(|(_idx, x), (_idy, y)| x.cmp(y)),
        }
        .unwrap()
        .0
    }
}

impl<T: Clone + Debug + Ord, I: Iterator<Item = T>> Iterator for MergeSortIterator<T, I> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.buffered.is_empty() {
            let idx = self.arg_cmp();
            let ret = if let Some(next) = self.sources.get_mut(&idx).unwrap().next() {
                self.buffered.insert(idx, next).unwrap()
            } else {
                self.buffered.remove(&idx).unwrap()
            };
            #[cfg(debug_assertions)]
            {
                // check ordering
                use Order::*;
                use Ordering::*;

                if self.last_elements.contains_key(&idx)
                    && match self
                        .last_elements
                        .insert(idx, ret.clone())
                        .unwrap()
                        .cmp(&ret)
                    {
                        Less => self.ordering == Desc,
                        Equal => false,
                        Greater => self.ordering == Asc,
                    }
                {
                    panic!("provided iterator is not ordered, last element: {:?}, current element: {:?}", self.last_elements[&idx], ret)
                }
            }
            Some(ret)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_sort_basic() {
        let i1 = vec![1u32, 6, 15];
        let i2 = vec![4u32, 9, 12];
        let i3 = vec![2u32, 8, 11];
        let i4 = vec![5u32, 7, 14];
        let i5 = vec![3u32, 10, 13];

        let iter = MergeSortIterator::new(
            vec![
                i1.into_iter(),
                i2.into_iter(),
                i3.into_iter(),
                i4.into_iter(),
                i5.into_iter(),
            ],
            Order::Asc,
        );
        assert_eq!(
            iter.collect::<Vec<u32>>(),
            vec![1u32, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
        );
    }

    #[test]
    fn test_merge_sort() {
        let mut vectors = Vec::with_capacity(1000);

        for _ in 0..199 {
            vectors.push(vec![]);
        }
        vectors.push(vec![1u32]);
        for _ in 200..399 {
            vectors.push(vec![]);
        }
        vectors.push(vec![4u32, 9, 12]);
        for _ in 400..599 {
            vectors.push(vec![]);
        }
        vectors.push(vec![2u32, 8, 11, 14]);
        for _ in 600..799 {
            vectors.push(vec![]);
        }
        vectors.push(vec![5u32, 7]);
        for _ in 800..999 {
            vectors.push(vec![]);
        }
        vectors.push(vec![3u32, 6, 10, 13, 15]);

        let iter = MergeSortIterator::new(
            vectors.into_iter().map(|v| v.into_iter()).collect(),
            Order::Asc,
        );
        assert_eq!(
            iter.collect::<Vec<u32>>(),
            vec![1u32, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
        );
    }
}
