use super::encoder_buffer::RlpBuffer;

#[derive(Default,Clone)]
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

    /// It appends the encoded bytes to dst.
    pub fn append_to_bytes(&self, dest: &mut Vec<u8>){
        let size = self.buffer.size(); // Allocate the capacity of the array as size
        
        self.buffer.copy_to(dest);

    }

    pub fn reset(&self) {
        unimplemented!()
    }
}
