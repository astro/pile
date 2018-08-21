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

pub enum SpiDevice {
    Invalid,
    Spi1(Spi<SPI1, (PA5<AF5>, PA6<AF5>, PB5<AF5>)>, dma2::S3),
    Spi2(Spi<SPI2, (PD3<AF5>, PC2<AF5>, PB15<AF5>)>, dma1::S4),
    Spi3(Spi<SPI3, (PC10<AF6>, PC11<AF6>, PC12<AF6>)>, dma1::S5),
}

impl SpiDevice {
    pub fn spi1(spi_dev: Spi<SPI1, (PA5<AF5>, PA6<AF5>, PB5<AF5>)>, dma_stream: dma2::S3) -> Self {
        SpiDevice::Spi1(spi_dev, dma_stream)
    }
    
    pub fn spi2(spi_dev: Spi<SPI2, (PD3<AF5>, PC2<AF5>, PB15<AF5>)>, dma_stream: dma1::S4) -> Self {
        SpiDevice::Spi2(spi_dev, dma_stream)
    }
    
    pub fn spi3(spi_dev: Spi<SPI3, (PC10<AF6>, PC11<AF6>, PC12<AF6>)>, dma_stream: dma1::S5) -> Self {
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
