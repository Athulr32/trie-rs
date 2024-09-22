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

pub fn compact_to_hex(compact: &[u8]) -> Vec<u8> {
    if compact.is_empty() {
        return Vec::new();
    }

    let mut base = keybytes_to_hex(compact);

    // delete terminator flag
    if base[0] < 2 {
        base.pop();
    }

    // apply odd flag
    let chop = 2 - (base[0] & 1);
    base[chop as usize..].to_vec()
}

fn keybytes_to_hex(str: &[u8]) -> Vec<u8> {
    let l = str.len() * 2 + 1;
    let mut nibbles = vec![0u8; l];
    
    for (i, &b) in str.iter().enumerate() {
        nibbles[i * 2] = b / 16;
        nibbles[i * 2 + 1] = b % 16;
    }
    
    nibbles[l - 1] = 16;
    nibbles
}

fn decode_nibbles(nibbles: &[u8], bytes: &mut [u8]) {
    for (bi, chunk) in nibbles.chunks(2).enumerate() {
        bytes[bi] = (chunk[0] << 4) | (chunk[1] & 0xF);
    }
}

pub fn has_term(s: &[u8]) -> bool {
    s.last().map_or(false, |&b| b == 16)
}
