use core::mem::replace;

use stm32f429::{SPI1, SPI2, SPI3};
use stm32f429_hal::gpio::{AF5, AF6};
use stm32f429_hal::gpio::gpioa::{PA5, PA6};
use stm32f429_hal::gpio::gpiob::{PB5, PB15};
use stm32f429_hal::gpio::gpioc::{PC2, PC10, PC11, PC12};
use stm32f429_hal::gpio::gpiod::PD3;
use stm32f429_hal::spi::Spi;
use stm32f429_hal::dma::Transfer;
use stm32f429_hal::dma::{dma1, dma2};

pub type Spi1Sck = PA5<AF5>;
pub type Spi1Miso = PA6<AF5>;
pub type Spi1Mosi = PB5<AF5>;
pub type Spi1Stream = dma2::S3;
pub type Spi2Sck = PD3<AF5>;
pub type Spi2Miso = PC2<AF5>;
pub type Spi2Mosi = PB15<AF5>;
pub type Spi2Stream = dma1::S4;
pub type Spi3Sck = PC10<AF6>;
pub type Spi3Miso = PC11<AF6>;
pub type Spi3Mosi = PC12<AF6>;
pub type Spi3Stream = dma1::S5;

pub enum SpiDevice {
    Invalid,
    Spi1(Spi<SPI1, (Spi1Sck, Spi1Miso, Spi1Mosi)>, Spi1Stream),
    Spi2(Spi<SPI2, (Spi2Sck, Spi2Miso, Spi2Mosi)>, Spi2Stream),
    Spi3(Spi<SPI3, (Spi3Sck, Spi3Miso, Spi3Mosi)>, Spi3Stream),
}

impl SpiDevice {
    pub fn spi1(spi_dev: Spi<SPI1, (Spi1Sck, Spi1Miso, Spi1Mosi)>, dma_stream: Spi1Stream) -> Self {
        SpiDevice::Spi1(spi_dev, dma_stream)
    }
    
    pub fn spi2(spi_dev: Spi<SPI2, (Spi2Sck, Spi2Miso, Spi2Mosi)>, dma_stream: Spi2Stream) -> Self {
        SpiDevice::Spi2(spi_dev, dma_stream)
    }
    
    pub fn spi3(spi_dev: Spi<SPI3, (Spi3Sck, Spi3Miso, Spi3Mosi)>, dma_stream: Spi3Stream) -> Self {
        SpiDevice::Spi3(spi_dev, dma_stream)
    }
    
    // Blocking write
    pub fn dma_write(&mut self, data: &[u8]) {
        let spi = replace(self, SpiDevice::Invalid);
        macro_rules! match_spi {
            ($($SPI: tt),+) => (
                match spi {
                    SpiDevice::Invalid => {}
                    $(
                        SpiDevice::$SPI(mut spi_dev, dma_stream) => {
                            let dma_stream = spi_dev.dma_write(dma_stream, data)
                                .wait()
                                .unwrap_or_else(|dma_stream| dma_stream);
                            replace(self, SpiDevice::$SPI(spi_dev, dma_stream));
                        }
                    )+
                }
            )
        }
        match_spi!(Spi1, Spi2, Spi3);
    }
}
