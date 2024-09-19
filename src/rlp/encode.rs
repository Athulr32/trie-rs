#[derive(Default,Clone)]
pub struct Listhead {
    pub offset: usize, // index of this header in string data
    pub size: usize,   // total size of encoded data (including list headers)
}

impl Listhead {
    pub fn encode(&self, buff: &[u8]) -> Vec<u8> {
        let headput = self.put_head(&mut buff.to_vec(), 0xC0, 0xF7, self.size as u64);
        return buff[..headput as usize].to_vec();
    }

    pub fn put_head(&self, buff: &mut Vec<u8>, smalltag: u8, largetag: u8, size: u64) -> isize {
        if size < 56 {
            buff[0] = smalltag + size as u8;
            return 1;
        }

        let sizesize = putint(&mut buff[1..], size);
        buff[0] = largetag + sizesize as u8;

        sizesize + 1
    }
}

/// putint writes i to the beginning of b in big endian byte
/// order, using the least number of bytes needed to represent i.
fn putint(b: &mut [u8], i: u64) -> isize {
    match i {
        0..=0xFF => {
            b[0] = i as u8;
            1
        }
        0x100..=0xFFFF => {
            b[0] = (i >> 8) as u8;
            b[1] = i as u8;
            2
        }
        0x1_0000..=0xFF_FFFF => {
            b[0] = (i >> 16) as u8;
            b[1] = (i >> 8) as u8;
            b[2] = i as u8;
            3
        }
        0x1_0000_00..=0xFFFF_FFFF => {
            b[0] = (i >> 24) as u8;
            b[1] = (i >> 16) as u8;
            b[2] = (i >> 8) as u8;
            b[3] = i as u8;
            4
        }
        0x1_0000_0000..=0xFF_FFFF_FFFF => {
            b[0] = (i >> 32) as u8;
            b[1] = (i >> 24) as u8;
            b[2] = (i >> 16) as u8;
            b[3] = (i >> 8) as u8;
            b[4] = i as u8;
            5
        }
        0x1_0000_0000_00..=0xFFFF_FFFF_FFFF => {
            b[0] = (i >> 40) as u8;
            b[1] = (i >> 32) as u8;
            b[2] = (i >> 24) as u8;
            b[3] = (i >> 16) as u8;
            b[4] = (i >> 8) as u8;
            b[5] = i as u8;
            6
        }
        0x1_0000_0000_0000..=0xFF_FFFF_FFFF_FFFF => {
            b[0] = (i >> 48) as u8;
            b[1] = (i >> 40) as u8;
            b[2] = (i >> 32) as u8;
            b[3] = (i >> 24) as u8;
            b[4] = (i >> 16) as u8;
            b[5] = (i >> 8) as u8;
            b[6] = i as u8;
            7
        }
        _ => {
            b[0] = (i >> 56) as u8;
            b[1] = (i >> 48) as u8;
            b[2] = (i >> 40) as u8;
            b[3] = (i >> 32) as u8;
            b[4] = (i >> 24) as u8;
            b[5] = (i >> 16) as u8;
            b[6] = (i >> 8) as u8;
            b[7] = i as u8;
            8
        }
    }
}
