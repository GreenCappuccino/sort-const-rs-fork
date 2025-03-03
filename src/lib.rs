#![no_std]

//! A macro for sorting arrays and slices at compile time.
//!
//! ## Usage
//! 
//! Just use the [`const_quicksort`] macro.
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

/// Used for our `call stack`
#[doc(hidden)]
pub use arrayvec_const::ArrayVec;

/// Sorts a `const` array or mutable slice as a `const` expression.
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
#[macro_export]
macro_rules! const_quicksort {
    ($data:expr) => {{
        macro_rules! cmp {
            ($a:expr, $b:expr) => {{ $a < $b }};
        }
        $crate::const_quicksort_adv!($data, cmp)
    }};
    ($data:expr, |$a:ident, $b:ident| $cmp:expr) => {{
        macro_rules! cmp {
            ($a_:expr, $b_:expr) => {{
                let ($a, $b) = (&$a_, &$b_);
                $cmp
            }};
        }
        $crate::const_quicksort_adv!($data, cmp)
    }};
    ($data:expr, $cmp:expr) => {{
        macro_rules! cmp {
            ($a:expr, $b:expr) => {{ $cmp(&$a, &$b) }};
        }
        $crate::const_quicksort_adv!($data, cmp)
    }};
}


/// Sorts a `const` array or mutable slice as a `const` expression.
/// 
/// The first parameter is the data to be sorted.
/// 
/// The second parameter is the path identifying the macro which should be invoked to compare elements `cmp!(a, b)`
#[macro_export]
macro_rules! const_quicksort_adv {
    ($data:expr, $cmp:path) => {{
        use core::mem::forget;

        struct Wrapper<T>(T);
        impl<T, const N: usize> Wrapper<[T; N]> {
            #[allow(unused)]
            const fn as_mut_slice(&mut self) -> &mut [T] {
                &mut self.0
            }
        }
        impl<'a, T> Wrapper<&'a mut [T]> {
            #[allow(unused)]
            const fn as_mut_slice(&mut self) -> &mut [T] {
                self.0
            }
        }
        impl<'a, T, const N: usize> Wrapper<&'a mut [T; N]> {
            #[allow(unused)]
            const fn as_mut_slice(&mut self) -> &mut [T] {
                self.0
            }
        }

        let mut data = Wrapper($data);
        // 2^64 elements should be plenty
        let mut slices = $crate::ArrayVec::<&mut [_], 64>::new();
        let Ok(()) = slices.try_push(data.as_mut_slice()) else { panic!("ran out of capacity") };
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
            let Ok(()) = slices.try_push(left) else { panic!("ran out of capacity") };
            if let Some((_pivot, right)) = right.split_first_mut() {
                let Ok(()) = slices.try_push(right) else { panic!("ran out of capacity") };
            }
        }
        forget(slices);

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
