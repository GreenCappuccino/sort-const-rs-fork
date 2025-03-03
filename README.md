# sort-const
[![Crates.io](https://img.shields.io/crates/v/sort-const.svg)](https://crates.io/crates/sort-const)
[![Workflow Status](https://github.com/Daniel-Aaron-Bloom/sort-const-rs/workflows/Rust/badge.svg)](https://github.com/Daniel-Aaron-Bloom/sort-const-rs/actions?query=workflow%3A%22Rust%22)

A macro for sorting arrays and slices at compile time.

### Usage

Just use the [`const_quicksort`] macro.

```rust
use sort_const::const_quicksort;

const fn lt(a: &u8, b: &u8) -> bool {
    *a < *b
}

const A: &[u8] = &const_quicksort!([3, 1, 2]);
const B: &[u8] = &const_quicksort!([3, 1, 2], |a, b| *a < *b);
const C: &[u8] = &const_quicksort!([3, 1, 2], lt);

assert_eq!(A, [1, 2, 3]);
assert_eq!(B, [1, 2, 3]);
assert_eq!(C, [1, 2, 3]);
```

Publication of this crate is blocked on [bluss/arrayvec#294](https://github.com/bluss/arrayvec/pull/294). Until then `git` dependency is the only option.

```toml
sort-const = { git = "https://github.com/Daniel-Aaron-Bloom/sort-const-rs" }
```

## License

Licensed under 
* MIT license ([LICENSE](LICENSE) or https://opensource.org/licenses/MIT)

[`const_quicksort`]: https://docs.rs/sort-const/latest/sort_const/macro.const_quicksort.html "macro sort_const::const_quicksort"
