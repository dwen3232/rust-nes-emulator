pub trait Memory {
    // Trait for byte addressability using 2-byte addresses
    fn read_byte(&self, index: u16) -> u8;

    fn write_byte(&mut self, index: u16, value: u8);

    fn read_two_bytes(&mut self, index: u16) -> u16 {
        let lsb = self.read_byte(index) as u16;
        let msb = self.read_byte(index + 1) as u16;
        let two_bytes = (msb << 8) + lsb;
        two_bytes
    }

    fn read_two_page_bytes(&mut self, index: u16) -> u16 {
        // same as read_two_bytes, but we loop back on the page boundary
        let lsb = self.read_byte(index) as u16;
        let msb = self.read_byte((index as u8).wrapping_add(1) as u16) as u16;
        let two_bytes = (msb << 8) + lsb;
        two_bytes
    }
}