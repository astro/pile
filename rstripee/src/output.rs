use embedded_hal::blocking::delay::DelayUs;
use stm32f429_hal::time::{KiloHertz, U32Ext};
use spi_dev::SpiDevice;
use ws2812_spi;

#[derive(Debug, Clone, Copy)]
pub enum Model {
    WS2801,
    // TODO: WS2812,
    SK6812RGBW,
}

impl Model {
    pub fn spi_clock(&self) -> KiloHertz {
        match self {
            Model::WS2801 =>
                2000u32.khz(),
            Model::SK6812RGBW =>
                (ws2812_spi::RESAMPLED_KHZ as u32).khz(),
        }
    }
}

pub struct Output {
    spi: SpiDevice,
    model: Model,
}

impl Output {
    pub fn new(spi: SpiDevice, model: Model) -> Self {
        Output { spi, model }
    }

    pub fn write<D: DelayUs<u16>>(&mut self, rgb: &[u8], delay: &mut D) {
        match self.model {
            Model::WS2801 => {
                self.spi.dma_write(rgb);
                delay.delay_us(500u16);
            }
            Model::SK6812RGBW => {
                let mut buf = [0u8; ws2812_spi::SAMPLERATE * 4 * 320];
                let output_len = buf.len()
                    .min(ws2812_spi::SAMPLERATE * 4 * buf.len() / 3);
                let output = ws2812_spi::TimedData::encode(
                    RgbToRgbw::new(rgb.iter().cloned()),
                    &mut buf[..]
                );
                self.spi.dma_write(&output.as_ref()[..output_len]);
                delay.delay_us(50u16);
            }
        }
    }
}

struct RgbToRgbw<I: Iterator<Item=u8>> {
    src: I,
    buf_pos: usize,
    buf: [u8; 4],
}

impl<I: Iterator<Item=u8>> RgbToRgbw<I> {
    fn new(src: I) -> Self {
        RgbToRgbw {
            src,
            buf_pos: 4,
            buf: [0u8; 4],
        }
    }
}

impl<I: Iterator<Item=u8>> Iterator for RgbToRgbw<I> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf_pos >= self.buf.len() {
            let r = self.src.next()?;
            let g = self.src.next()?;
            let b = self.src.next()?;
            let w = r.min(g).min(b);
            self.buf = [r - w, g - w, b - w, w];
            self.buf_pos = 0;
        }

        let buf_pos = self.buf_pos;
        self.buf_pos += 1;
        Some(self.buf[buf_pos])
    }
}
