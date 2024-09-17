pub fn prefix_len(a: &Vec<u8>, b: &Vec<u8>) -> usize {
    let length = if a.len() > b.len() { b.len() } else { a.len() };
    let mut i = 0;

    while i < length {
        if a[i] != b[i] {
            break;
        }

        i = i + 1;
    }

    i
}
