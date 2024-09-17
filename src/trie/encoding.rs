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

pub fn hex_to_compact(hex: &[u8]) -> Vec<u8> {
    let mut terminator = 0u8;
    let hex_len = if has_term(hex) {
        terminator = 1;
        hex.len() - 1
    } else {
        hex.len()
    };

    let mut buf = vec![0u8; hex_len / 2 + 1];
    buf[0] = terminator << 5; // the flag byte

    if hex_len & 1 == 1 {
        buf[0] |= 1 << 4; // odd flag
        buf[0] |= hex[0] & 0xF; // first nibble is contained in the first byte
        decode_nibbles(&hex[1..hex_len], &mut buf[1..]);
    } else {
        decode_nibbles(&hex[..hex_len], &mut buf[1..]);
    }

    buf
}

fn decode_nibbles(nibbles: &[u8], bytes: &mut [u8]) {
    for (bi, chunk) in nibbles.chunks(2).enumerate() {
        bytes[bi] = (chunk[0] << 4) | (chunk[1] & 0xF);
    }
}

fn has_term(s: &[u8]) -> bool {
    s.last().map_or(false, |&b| b == 16)
}