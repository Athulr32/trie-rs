/// Splits a Vector into the content of a list and any remaining bytes after the list.
/// This function first attempts to split the input byte slice using the `split` function.
/// If the resulting `Kind` is a `List`, it returns the content of the list and the
/// remaining bytes. Otherwise, it returns an error.
pub fn split_list(buff: &[u8]) -> Result<(&[u8], &[u8]), ()> {
    let (kind, content, rest) = split(&buff[..])?;

    if kind != Kind::List {
        return Err(());
    }

    Ok((content, rest))
}

/// This function parses the input as RLP (Recursive Length Prefix) encoded data,
/// extracts the first value, and returns it along with any remaining data.
fn split(buff: &[u8]) -> Result<(Kind, &[u8], &[u8]), ()> {
    let (k, ts, cs) = read_kind(buff)?;

    let content_end = (ts + cs) as usize;
    if content_end > buff.len() {
        return Err(());
    }

    Ok((
        k,
        &buff[ts as usize..content_end],
        &buff[content_end as usize..],
    ))
}

/// CountValues counts the number of encoded values in b.
pub fn count_values(mut buff: &[u8]) -> Result<u8, ()> {
    let mut i = 0;
    while !buff.is_empty() {
        let (_, tagsize, size) = read_kind(buff)?;

        let index = (tagsize + size) as usize;
        if index > buff.len() {
            return Err(());
        }
        buff = &buff[index..];
        i += 1;
    }

    Ok(i)
}

/// This function assumes the input is RLP (Recursive Length Prefix) encoded data representing a string.
/// It extracts the content of the string and separates it from any trailing data.
pub fn split_string(buff: &[u8]) -> Result<(&[u8], &[u8]), ()> {
    let (k, content, rest) = split(buff)?;

    if k == Kind::List {
        return Err(());
    }

    Ok((content, rest))
}

#[derive(PartialEq)]
pub enum Kind {
    Byte,
    String,
    List,
}

fn read_kind(buff: &[u8]) -> Result<(Kind, u64, u64), ()> {
    if buff.len() == 0 {
        return Err(());
    }

    let b = buff[0];

    let (k, tagsize, contentsize) = match b {
        b if b < 0x80 => (Kind::Byte, 0, 1),
        b if b < 0xB8 => {
            let contentsize = (b - 0x80) as u64;
            // Reject strings that should've been single bytes.
            if contentsize == 1 && buff.len() > 1 && buff[1] < 128 {
                return Err(());
            }
            (Kind::String, 1, contentsize)
        }
        b if b < 0xC0 => {
            let tagsize = (b - 0xB7 + 1) as u64;
            let contentsize = read_size(&buff[1..], b - 0xB7)?;
            (Kind::String, tagsize, contentsize)
        }
        b if b < 0xF8 => (Kind::List, 1, (b - 0xC0) as u64),
        _ => {
            let tagsize = (b - 0xF7 + 1) as u64;
            let contentsize = read_size(&buff[1..], b - 0xF7)?;
            (Kind::List, tagsize, contentsize)
        }
    };

    Ok((k, tagsize, contentsize))
}

fn read_size(b: &[u8], slen: u8) -> Result<u64, ()> {
    if b.len() < slen as usize {
        return Err(());
    }

    let s = match slen {
        1 => u64::from(b[0]),
        2 => u64::from(b[0]) << 8 | u64::from(b[1]),
        3 => u64::from(b[0]) << 16 | u64::from(b[1]) << 8 | u64::from(b[2]),
        4 => u64::from(b[0]) << 24 | u64::from(b[1]) << 16 | u64::from(b[2]) << 8 | u64::from(b[3]),
        5 => {
            u64::from(b[0]) << 32
                | u64::from(b[1]) << 24
                | u64::from(b[2]) << 16
                | u64::from(b[3]) << 8
                | u64::from(b[4])
        }
        6 => {
            u64::from(b[0]) << 40
                | u64::from(b[1]) << 32
                | u64::from(b[2]) << 24
                | u64::from(b[3]) << 16
                | u64::from(b[4]) << 8
                | u64::from(b[5])
        }
        7 => {
            u64::from(b[0]) << 48
                | u64::from(b[1]) << 40
                | u64::from(b[2]) << 32
                | u64::from(b[3]) << 24
                | u64::from(b[4]) << 16
                | u64::from(b[5]) << 8
                | u64::from(b[6])
        }
        8 => {
            u64::from(b[0]) << 56
                | u64::from(b[1]) << 48
                | u64::from(b[2]) << 40
                | u64::from(b[3]) << 32
                | u64::from(b[4]) << 24
                | u64::from(b[5]) << 16
                | u64::from(b[6]) << 8
                | u64::from(b[7])
        }
        _ => return Err(()),
    };

    // Reject sizes < 56 (shouldn't have separate size) and sizes with
    // leading zero bytes.
    if s < 56 || b[0] == 0 {
        return Err(());
    }

    Ok(s)
}




pub fn append_uint64(mut b: Vec<u8>, i: u64) -> Vec<u8> {
    if i == 0 {
        b.push(0x80);
    } else if i < 128 {
        b.push(i as u8);
    } else {
        match i {
            1..=0xFF => {
                b.extend_from_slice(&[0x81, i as u8]);
            }
            0x100..=0xFFFF => {
                b.push(0x82);
                b.extend_from_slice(&i.to_be_bytes()[6..]);
            }
            0x1_0000..=0xFF_FFFF => {
                b.push(0x83);
                b.extend_from_slice(&i.to_be_bytes()[5..]);
            }
            0x100_0000..=0xFFFF_FFFF => {
                b.push(0x84);
                b.extend_from_slice(&i.to_be_bytes()[4..]);
            }
            0x1_0000_0000..=0xFFFF_FFFF_FF => {
                b.push(0x85);
                b.extend_from_slice(&i.to_be_bytes()[3..]);
            }
            0x100_0000_0000..=0xFFFF_FFFF_FFFF => {
                b.push(0x86);
                b.extend_from_slice(&i.to_be_bytes()[2..]);
            }
            0x1_0000_0000_0000..=0xFFFF_FFFF_FFFF_FF => {
                b.push(0x87);
                b.extend_from_slice(&i.to_be_bytes()[1..]);
            }
            _ => {
                b.push(0x88);
                b.extend_from_slice(&i.to_be_bytes());
            }
        }
    }
    b
}