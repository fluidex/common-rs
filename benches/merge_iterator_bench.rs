use criterion::*;
use fluidex_common::helper::{MergeSortIterator, Order};
use rand::prelude::*;
use rand::seq::index;

fn generate_input() -> Vec<Vec<i32>> {
    let mut vectors = Vec::with_capacity(1000);
    vectors.resize(1000, vec![]);
    let indexes = index::sample(&mut thread_rng(), 1000, 10).into_vec();
    for i in 0..1000 {
        let selected = indexes.choose(&mut thread_rng()).unwrap();
        vectors[*selected].push(i);
    }
    vectors
}

fn criterion_benchmark(c: &mut Criterion) {
    let data = generate_input();
    c.bench_function("test_merge_sort_hashmap", |b| {
        b.iter_batched(
            || data.clone().into_iter().map(|v| v.into_iter()).collect(),
            |vectors| MergeSortIterator::new(vectors, Order::Asc).for_each(drop),
            BatchSize::SmallInput,
        )
    });
    c.bench_function("test_merge_sort_origin", |b| {
        b.iter_batched(
            || data.clone().into_iter().map(|v| v.into_iter()).collect(),
            |vectors| {
                old_impl::MergeSortIterator::new(vectors, old_impl::Order::Asc).for_each(drop)
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

mod old_impl {
    use std::cmp::Ordering;
    use std::fmt::Debug;

    /// A Iterator that run merge sort on `N` ordered iterators.
    #[derive(Debug)]
    pub struct MergeSortIterator<T, I> {
        sources: Vec<I>,
        buffered: Vec<Option<T>>,
        #[cfg(debug_assertions)]
        last_elements: Vec<Option<T>>,
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
            let size = sources.len();
            let mut iter = Self {
                sources,
                buffered: Vec::with_capacity(size),
                #[cfg(debug_assertions)]
                last_elements: Vec::with_capacity(size),
                ordering,
            };
            iter.buffered.resize(size, None);
            #[cfg(debug_assertions)]
            iter.last_elements.resize(size, None);
            for (i, sub_iter) in iter.sources.iter_mut().enumerate() {
                iter.buffered[i] = sub_iter.next();
            }
            iter
        }

        /// Find the most `ordering` element in the `buffered` array.
        /// ## panics
        /// When the elements in `buffered` are all `None`, calling to this function will panics.
        fn arg_cmp(&mut self) -> usize {
            use Order::*;

            let tmp = self
                .buffered
                .iter()
                .enumerate()
                .filter(|(_id, x)| x.is_some());
            match self.ordering {
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
            if self.buffered.iter().any(|x| x.is_some()) {
                let idx = self.arg_cmp();
                let ret = if let Some(next) = self.sources.get_mut(idx).unwrap().next() {
                    self.buffered[idx].replace(next).unwrap()
                } else {
                    self.buffered[idx].take().unwrap()
                };
                #[cfg(debug_assertions)]
                {
                    // check ordering
                    use Order::*;
                    use Ordering::*;

                    if self.last_elements[idx].is_some()
                        && match self.last_elements[idx].as_ref().unwrap().cmp(&ret) {
                            Less => self.ordering == Desc,
                            Equal => false,
                            Greater => self.ordering == Asc,
                        }
                    {
                        panic!("provided iterator is not ordered, last element: {:?}, current element: {:?}", self.last_elements[idx], ret)
                    }
                    self.last_elements[idx] = Some(ret.clone());
                }
                Some(ret)
            } else {
                None
            }
        }
    }
}
