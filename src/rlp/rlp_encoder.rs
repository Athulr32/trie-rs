use super::encoder_buffer::RlpBuffer;

pub struct RlpEncoder {
    pub buffer: RlpBuffer,
}

impl RlpEncoder {
    pub fn write(&mut self, bytes: Vec<u8>) -> usize {
        self.buffer.write(bytes)
    }

    pub fn list(&mut self) -> usize {
        self.buffer.list()
    }

    pub fn list_end(&mut self, index: usize) {
        self.buffer.list_end(index);
    }

    pub fn write_bytes(&mut self, bytes: Vec<u8>) {
        self.buffer.write_bytes(bytes);
    }
}
