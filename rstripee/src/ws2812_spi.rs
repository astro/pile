use core::convert::AsRef;

pub const SAMPLERATE: usize = 12;
const RATE_KHZ: usize = 800;
pub const RESAMPLED_KHZ: usize = SAMPLERATE * RATE_KHZ;
const D_TIME: usize = 1_000_000 / 800;
const D0H: usize = SAMPLERATE * 350 / D_TIME;
const D0L: usize = SAMPLERATE * 700 / D_TIME;
const D1H: usize = SAMPLERATE * 800 / D_TIME;
const D1L: usize = SAMPLERATE * 600 / D_TIME;


pub struct TimedData<'a> {
    buf: &'a mut [u8],
    bits: usize,
}

impl<'a> TimedData<'a> {
    /// `output` should be nulled
    pub fn encode(input: &[u8], output: &'a mut [u8]) -> Self {
        let mut self_ = TimedData { buf: output, bits: 0 };

        for byte in input {
            for bit_pos in 0..8 {
                let bit = byte & (0x80 >> bit_pos) != 0;
                if ! bit {
                    self_.push(true, D0H);
                    self_.push(false, D0L);
                } else {
                    self_.push(true, D1H);
                    self_.push(false, D1L);
                }
            }
        }

        self_
    }

    fn push(&mut self, bit: bool, len: usize) {
        for _ in 0..len {
            self.push_bit(bit);
        }
    }

    fn push_bit(&mut self, bit: bool) {
        let pos = self.bits >> 3;
        let bit_pos = self.bits & 7;

        let mask = 0x80 >> bit_pos;
        if bit {
            self.buf.as_mut()[pos] |= mask;
        } else {
            self.buf.as_mut()[pos] &= !mask;
        }

        self.bits += 1;
    }

    pub fn len(&self) -> usize {
        (self.bits | 7) + 1
    }

    pub fn into_inner(self) -> &'a mut [u8] {
        self.buf
    }
}

impl<'a> AsRef<[u8]> for TimedData<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.buf.as_ref()[0..self.len()]
    }
}
