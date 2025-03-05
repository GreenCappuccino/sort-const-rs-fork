use sort_const::{const_quicksort, const_shellsort};

const ARRAY: [u8; 10] = [1, 2, 3, 6, 5, 4, 7, 8, 9, 0];
const QUICK_SORTED_ARRAY: [u8; 10] = const_quicksort!(ARRAY);
const SHELL_SORTED_ARRAY: [u8; 10] = const_shellsort!(ARRAY);

fn main() {
    assert_eq!(QUICK_SORTED_ARRAY, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(SHELL_SORTED_ARRAY, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}
