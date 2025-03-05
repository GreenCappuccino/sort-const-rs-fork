use sort_const::const_shellsort;

const SHELL_SORTED_ARRAY: &[u32] = &{
    let mut data = [0_u32; 20_000];
    let mut i = 0;
    while i < data.len() {
        if i & 1 == 0 {
            data[i] = i as u32;
        }
        i += 1;
    }
    const_shellsort!(&mut data);
    data
};

fn main() {}
