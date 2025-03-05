#![no_std]

//! A macro for sorting arrays and slices at compile time.
//!
//! ## Usage
//!
//! Just use the [`const_quicksort`] or [`const_shellsort`] macros.
//!
//! ```
//! use sort_const::const_quicksort;
//!
//! const fn lt(a: &u8, b: &u8) -> bool {
//!     *a < *b
//! }
//!
//! const A: &[u8] = &const_quicksort!([3, 1, 2]);
//! const B: &[u8] = &const_quicksort!([3, 1, 2], |a, b| *a < *b);
//! const C: &[u8] = &const_quicksort!([3, 1, 2], lt);
//!
//! assert_eq!(A, [1, 2, 3]);
//! assert_eq!(B, [1, 2, 3]);
//! assert_eq!(C, [1, 2, 3]);
//! ```

use core::mem::forget;

/// Used for our `call stack`
#[doc(hidden)]
pub use arrayvec_const::ArrayVec;

/// A helper to turn everything into mutable slices
#[doc(hidden)]
pub struct Wrapper<T>(pub T);
impl<T, const N: usize> Wrapper<[T; N]> {
    #[inline(always)]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.0
    }
}
impl<T> Wrapper<&'_ mut [T]> {
    #[inline(always)]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        self.0
    }
}
impl<T, const N: usize> Wrapper<&'_ mut [T; N]> {
    #[inline(always)]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        self.0
    }
}
impl<T, const N: usize> Wrapper<&'_ [T; N]> {
    #[inline(always)]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        panic!("Cannot sort non-mut `&`. Did you mean to use `&mut`?")
    }
}

#[doc(hidden)]
#[track_caller]
#[inline]
pub const fn expect_push<T, const LEN: usize>(v: &mut ArrayVec<T, LEN>, element: T) {
    let res = v.try_push(element);
    if res.is_err() {
        panic!("ran out of capacity");
    }
    forget(res);
}

/// Some nice gaps for shellsort
#[doc(hidden)]
pub const A366726: [usize; 32] = [
    1,
    4,
    9,
    20,
    45,
    102,
    230,
    516,
    1158,
    2599,
    5831,
    13082,
    29351,
    65853,
    147748,
    331490,
    743735,
    1668650,
    3743800,
    8399623,
    18845471,
    42281871,
    94863989,
    212837706,
    477524607,
    1071378536,
    2403754591,
    5393085583,
    12099975682,
    27147615084,
    60908635199,
    136655165852,
];

/// Sorts a `const` array or mutable slice as a `const` expression using quicksort.
///
/// Can be called with just the data to be sorted, in which case elements are compared by `a < b` resulting in the smallest element at
/// the head of the data and the largest at the tail.
///
/// ```
/// use sort_const::const_quicksort;
///
/// const U8S: &[u8] = &const_quicksort!([3, 1, 2]);
///
/// assert_eq!(U8S, [1, 2, 3]);
/// ```
///
/// Can also be called with a lambda-like expression (e.g. `|a, b| a < b`) where `true` identifies that `a` should come before `b`.
///
/// ```
/// use sort_const::const_quicksort;
///
/// #[derive(Debug, PartialEq)]
/// struct Foo(u8);
///
/// const FOOS_MUT_REF: &[Foo] = &{
///     let mut foo = [Foo(1), Foo(2), Foo(4), Foo(3)];
///     const_quicksort!(foo.split_last_mut().expect("failed").1, |a, b| a.0 > b.0);
///     foo
/// };
///
/// assert_eq!(FOOS_MUT_REF, [4, 2, 1, 3].map(Foo));
/// ```
///
/// Can also be called with the name of a `const` function which must return a boolean and which will be evaluated over `&data[a], &data[b]`.
///
/// ```
/// use sort_const::const_quicksort;
///
/// #[derive(Debug, PartialEq)]
/// struct Foo(u8);
///
/// const fn gt(lhs: &Foo, rhs: &Foo) -> bool {
///     lhs.0 > rhs.0
/// }
/// const FOOS_MUT_REF: &[Foo] = &{
///     let mut foo = [Foo(1), Foo(2), Foo(4), Foo(3)];
///     const_quicksort!(foo.split_last_mut().expect("failed").1, gt);
///     foo
/// };
///
/// assert_eq!(FOOS_MUT_REF, [4, 2, 1, 3].map(Foo));
/// ```
///
/// The `@depth` parameter should only be used if you encounter a scenario where "stack overflows" start occuring.
/// ```compile_fail
/// use sort_const::const_quicksort;
///
/// const SORTED_ARRAY: &[u32] = &{
///     let mut data = [0_u32; 1000];
///     let mut i = 0;
///     while i < data.len() {
///         if i & 1 == 0 {
///             data[i] = i as u32;
///         }
///         i += 1;
///     }
///     const_quicksort!(@8, &mut data);
///     data
/// };
/// ```
///
#[macro_export]
macro_rules! const_quicksort {
    ($(@$depth:literal,)? $data:expr) => {{
        macro_rules! cmp {
            ($a:expr, $b:expr) => {{ $a < $b }};
        }
        $crate::const_quicksort_adv!($(@$depth,)? $data, cmp)
    }};
    ($(@$depth:literal,)? $data:expr, |$a:ident, $b:ident| $cmp:expr) => {{
        macro_rules! cmp {
            ($a_:expr, $b_:expr) => {{
                let ($a, $b) = (&$a_, &$b_);
                $cmp
            }};
        }
        $crate::const_quicksort_adv!($(@$depth,)? $data, cmp)
    }};
    ($(@$depth:literal,)? $data:expr, $cmp:expr) => {{
        macro_rules! cmp {
            ($a:expr, $b:expr) => {{ $cmp(&$a, &$b) }};
        }
        $crate::const_quicksort_adv!($(@$depth,)? $data, cmp)
    }};
}

/// Sorts a `const` array or mutable slice as a `const` expression using quicksort.
///
/// The optional `@` prefix parameter is the max stack size for recursion.
///
/// The first parameter is the data to be sorted.
///
/// The second parameter is the path identifying the macro which should be invoked to compare elements `cmp!(a, b)`
#[macro_export]
macro_rules! const_quicksort_adv {
    ($data:expr, $cmp:path) => { $crate::const_quicksort_adv!(@1024, $data, $cmp) };
    (@$depth:literal, $data:expr, $cmp:path) => {{
        let mut data = $crate::Wrapper($data);
        let mut slices = $crate::ArrayVec::<&mut [_], $depth>::new();
        $crate::expect_push(&mut slices, data.as_mut_slice());
        while let Some(data) = slices.pop() {
            match data.len() {
                0 | 1 => continue,
                2 => {
                    if !{$cmp!(data[0], data[1])} {
                        data.swap(0, 1);
                    }
                    continue;
                },
                _ => {}
            }


            let (pivot, rest) = data
                .split_first_mut()
                .expect("slice is not empty, as verified above");

            let mut left = 0;
            let mut right = rest.len() - 1;
            while left <= right {
                if {$cmp!(rest[left], *pivot)} {
                    left += 1;
                } else if !{$cmp!(rest[right], *pivot)} {
                    if right == 0 {
                        break;
                    }
                    right -= 1;
                } else {
                    rest.swap(left, right);
                    left += 1;
                    if right == 0 {
                        break;
                    }
                    right -= 1;
                }
            }

            data.swap(0, left);
            let (left, right) = data.split_at_mut(left);
            match right.split_first_mut() {
                None => {
                    $crate::expect_push(&mut slices, left);
                }
                Some((_pivot, right)) if left.len() >= right.len() => {
                    $crate::expect_push(&mut slices, left);
                    $crate::expect_push(&mut slices, right);
                }
                Some((_pivot, right)) => {
                    $crate::expect_push(&mut slices, right);
                    $crate::expect_push(&mut slices, left);
                }
            }
        }
        ::core::mem::forget(slices);

        data.0
    }};
}

/// Sorts a `const` array or mutable slice as a `const` expression using quicksort.
///
/// Can be called with just the data to be sorted, in which case elements are compared by `a < b` resulting in the smallest element at
/// the head of the data and the largest at the tail.
///
/// ```
/// use sort_const::const_shellsort;
///
/// const U8S: &[u8] = &const_shellsort!([3, 1, 2]);
///
/// assert_eq!(U8S, [1, 2, 3]);
/// ```
///
/// Can also be called with a lambda-like expression (e.g. `|a, b| a < b`) where `true` identifies that `a` should come before `b`.
///
/// ```
/// use sort_const::const_shellsort;
///
/// #[derive(Debug, PartialEq)]
/// struct Foo(u8);
///
/// const FOOS_MUT_REF: &[Foo] = &{
///     let mut foo = [Foo(1), Foo(2), Foo(4), Foo(3)];
///     const_shellsort!(foo.split_last_mut().expect("failed").1, |a, b| a.0 > b.0);
///     foo
/// };
///
/// assert_eq!(FOOS_MUT_REF, [4, 2, 1, 3].map(Foo));
/// ```
///
/// Can also be called with the name of a `const` function which must return a boolean and which will be evaluated over `&data[a], &data[b]`.
#[macro_export]
macro_rules! const_shellsort {
    ($(@$seq:expr,)? $data:expr) => {{
        macro_rules! cmp {
            ($a:expr, $b:expr) => {{ $a < $b }};
        }
        $crate::const_shellsort_adv!($(@$seq,)? $data, cmp)
    }};
    ($(@$seq:expr,)? $data:expr, |$a:ident, $b:ident| $cmp:expr) => {{
        macro_rules! cmp {
            ($a_:expr, $b_:expr) => {{
                let ($a, $b) = (&$a_, &$b_);
                $cmp
            }};
        }
        $crate::const_shellsort_adv!($(@$seq,)? $data, cmp)
    }};
    ($(@$seq:expr,)? $data:expr, $cmp:expr) => {{
        macro_rules! cmp {
            ($a:expr, $b:expr) => {{ $cmp(&$a, &$b) }};
        }
        $crate::const_shellsort_adv!($(@$seq,)? $data, cmp)
    }};
}

/// Sorts a `const` array or mutable slice as a `const` expression using shellsort.
///
/// The optional `@` prefix parameter is the sort.
///
/// The first parameter is the data to be sorted.
///
/// The second parameter is the path identifying the macro which should be invoked to compare elements `cmp!(a, b)`
#[macro_export]
macro_rules! const_shellsort_adv {
    ($data:expr, $cmp:path) => { $crate::const_shellsort_adv!(@$crate::A366726, $data, $cmp) };
    (@$seq:expr, $data:expr, $cmp:path) => {{
        const GAPS: &[usize] = &$seq;
        const GAPS_LEN: usize = GAPS.len();

        let mut data = $crate::Wrapper($data);

        let arr = data.as_mut_slice();
        let n = arr.len();
        let mut gaps = {
            let mut gaps = $crate::ArrayVec::<usize, GAPS_LEN>::new();
            let mut i = 0;
            while i < GAPS_LEN && GAPS[i] < arr.len() {
                $crate::expect_push(&mut gaps, GAPS[i]);
                i += 1;
            }
            gaps
        };

        // Start with the largest gap and work down to a gap of 1
        while let Some(gap) = gaps.pop() {
            // Do a gapped insertion sort for every element in gaps
            // Each loop leaves a[0..gap-1] in gapped order
            let mut i = gap;
            while i < arr.len() {
                // save a[i] in temp and make a hole at position i
                // This is safe because we are holding a temporary over a series of swaps.
                let temp = unsafe { ::core::ptr::read(&arr[i]) };

                // shift earlier gap-sorted elements up until the correct location for a[i] is found
                let mut j = i;
                while j >= gap && {$cmp!(temp, arr[j - gap])} {
                    // This is safe because the previous item is either saved away in `temp` or
                    // was previously copied to it's new location
                    unsafe { ::core::ptr::copy(&arr[j - gap], &mut arr[j], 1); }
                    j -= gap
                }

                // put temp (the original a[i]) in its correct location
                // This is safe because we writing the last swap.
                unsafe { ::core::ptr::write(&mut arr[j], temp); }
                i += 1;
            }
        }
        core::mem::forget(gaps);
        data.0
    }};
}

#[cfg(test)]
mod test {
    #[derive(Debug)]
    struct Foo(u8);

    const fn lt(a: &u8, b: &u8) -> bool {
        *a < *b
    }

    const U8S: &[u8] = &const_quicksort!([3, 2, 1]);
    const U8S_FN: &[u8] = &const_quicksort!([3, 2, 1], lt);
    const FOOS: &[Foo] = &const_quicksort!([Foo(1), Foo(2), Foo(4), Foo(3)], |a, b| a.0 > b.0);
    const FOOS_MUT_REF: &[Foo] = &{
        let mut foo = [Foo(1), Foo(2), Foo(4), Foo(3)];
        const_quicksort!(foo.split_last_mut().expect("failed").1, |a, b| a.0 > b.0);
        foo
    };

    #[test]
    fn test_u8() {
        assert_eq!(U8S, [1, 2, 3]);
    }

    #[test]
    fn test_u8_fn() {
        assert_eq!(U8S_FN, [1, 2, 3]);
    }

    #[test]
    fn test_foo() {
        assert!(FOOS.iter().map(|v| v.0).eq([4, 3, 2, 1]));
        assert!(FOOS_MUT_REF.iter().map(|v| v.0).eq([4, 2, 1, 3]));
    }
}
