use sort_const::const_quicksort;

const QUICK_SORTED_ARRAY: &[u32] = &{
    let mut data = [0_u32; 3625];
    let mut i = 0;
    while i < data.len() {
        if i & 1 == 0 {
            data[i] = i as u32;
        }
        i += 1;
    }
    const_quicksort!(@12, &mut data);
    data
};

fn main() {}
