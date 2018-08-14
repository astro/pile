pub struct MacAddrGenerator {
    pos: usize,
    mac: [u8; 6],
}

impl MacAddrGenerator {
    pub fn new() -> Self {
        MacAddrGenerator {
            pos: 0,
            mac: [0x02, 0, 0, 0, 0, 0],
        }
    }

    pub fn feed<I: Iterator<Item=u8>>(&mut self, src: I) {
        for b in src {
            if self.pos == 0 {
                // Retain flag for private MAC addresses
                self.mac[0] =
                    ((self.mac[0] ^ b) & 0xFC) |
                    0b10;
            } else {
                self.mac[self.pos] ^= b;
            }

            self.pos += 1;
            if self.pos >= self.mac.len() {
                self.pos = 0;
            }
        }
    }

    pub fn into_addr(self) -> [u8; 6] {
        self.mac
    }
}
