pub const EMPTY_STRING: [u8; 1] = [0x80];

pub struct Listhead {
    pub offset: usize, // index of this header in string data
    pub size: usize,   // total size of encoded data (including list headers)
}

pub struct RlpBuffer {
    pub str: Vec<u8>,          // string data, contains everything except list headers
    pub lheads: Vec<Listhead>, // all list headers
    pub lhsize: usize,         // sum of sizes of all encoded list headers
    pub size_buf: [u8; 9],     // auxiliary buffer for uint encoding
}

impl RlpBuffer {
    pub fn list(&mut self) -> usize {
        self.lheads.push(Listhead {
            offset: self.str.len(),
            size: self.lhsize,
        });

        self.lheads.len() - 1
    }

    pub fn list_end(&mut self, index: usize) {
        let size = self.size();
        let lh = &mut self.lheads[index];

        lh.size = size - lh.offset - lh.size;

        if lh.size < 56 {
            self.lhsize += 1; // length encoded into kind tag
        } else {
            self.lhsize += 1 + lh.size;
        }
    }

    pub fn size(&self) -> usize {
        return self.str.len() + self.lhsize;
    }

    pub fn write_bytes(&mut self, bytes: Vec<u8>) {
        if bytes.len() == 1 && bytes[0] <= 0x7F {
            // fits single byte, no string header
            self.str.push(bytes[0]);
        } else {
            self.encode_string_header(bytes.len());
            self.str = [self.str.clone(), bytes].concat();
        }
    }

    pub fn write_string(&mut self, s: String) {
        self.write_bytes(s.as_bytes().to_vec());
    }

    pub fn write(&mut self, bytes: Vec<u8>) -> usize {
        let len = bytes.len();
        self.str = [self.str.clone(), bytes].concat();

        len
    }

    pub fn encode_string_header(&self, size: usize) {
        unimplemented!()
    }
}
