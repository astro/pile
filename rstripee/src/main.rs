#![no_std]
#![feature(used)]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate panic_itm;
extern crate cortex_m_semihosting;
#[macro_use(exception, interrupt)]
extern crate stm32f429;
extern crate stm32f429_hal;
extern crate embedded_hal;
#[macro_use(block)]
extern crate nb;

use cortex_m::asm;
use stm32f429::{Peripherals, CorePeripherals, SYST, SCB, SPI1};
use stm32f429_hal::flash::FlashExt;
use stm32f429_hal::rcc::RccExt;
use stm32f429_hal::gpio::GpioExt;
use stm32f429_hal::time::U32Ext;
use stm32f429_hal::delay::Delay;
use stm32f429_hal::spi::Spi;
use embedded_hal::digital::OutputPin;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::spi::Write as SpiWrite;
use embedded_hal::spi;
use embedded_hal::spi::FullDuplex;

use core::fmt::Write;
use cortex_m_semihosting::hio;

fn send_colors<SPI: SpiWrite<u8>>(spi: &mut SPI, colors: &[[u8; 3]]) -> Result<(), SPI::Error> {
    for rgb in colors {
        // for c in rgb {
            // block!(spi.send(*c))?;
            // block!(spi.read())?;
        // }
            spi.write(rgb)?;
    }

    Ok(())
}

fn main() {
    // let mut stdout = hio::hstdout().unwrap();

    let p = Peripherals::take().unwrap();
    let mut cp = CorePeripherals::take().unwrap();

    let mut scb = cp.SCB;
    if ! SCB::icache_enabled() {
        // writeln!(stdout, "Enable I-Cache").unwrap();
        scb.enable_icache();
    }
    if ! SCB::dcache_enabled() {
        // writeln!(stdout, "Enable D-Cache").unwrap();
        let cpuid = &mut cp.CPUID;
        scb.enable_dcache(cpuid);
    }
    
    let mut rcc = p.RCC.constrain();
    
    let mut gpiob = p.GPIOB.split(&mut rcc.ahb1);
    let mut gpioc = p.GPIOC.split(&mut rcc.ahb1);
    let mut led_green = gpiob.pb0.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let mut led_blue = gpiob.pb7.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let mut led_red = gpiob.pb14.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    led_red.set_high();
    
    // TRY the other clock configuration
    let mut flash = p.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // let clocks = rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz()).freeze(&mut flash.acr);
    // writeln!(stdout, "clocks: {:?}", clocks);

    let mut delay = Delay::new(cp.SYST, clocks);

    let mosi = gpiob.pb15.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
    let miso = gpioc.pc2.into_af5(&mut gpioc.moder, &mut gpioc.afrl);
    let sck = gpiob.pb13.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
    let _nss = gpiob.pb12.into_af5(&mut gpiob.moder, &mut gpiob.afrh);

    let spi_mode = spi::Mode {
        polarity: spi::Polarity::IdleLow,
        phase: spi::Phase::CaptureOnFirstTransition,
    };
    let mut spi = Spi::spi2(p.SPI2, (sck, miso, mosi), spi_mode, 2.mhz(), clocks, &mut rcc.apb1);
    led_red.set_low();

    let mut colors = [[0u8; 3]; 160];
    let colors_len = colors.len();
    let mut offset = 0usize;
    loop {
        offset += 1;

        for (x, rgb) in colors.iter_mut().enumerate() {
            let mut x1 = x as f32 + offset as f32 / 10.0;
            while x1 >= colors_len as f32 {
                x1 -= colors_len as f32;
            }
            if x1 >= 0.0 && x1 < 16.0 {
                fn abs(x: f32) -> f32 {
                    if x < 0.0 { - x } else { x }
                }
                let mut d = abs(x1 - 8.0) / 8.0;
                if d < 0.0 { d = 0.0; }
                if d > 1.0 { d = 1.0; }
                *rgb = [(255.0 * d) as u8, 63 - (63.0 * d) as u8, (255.0 * d) as u8];
            } else {
                *rgb = [0, 15, 0];
            }
        }
        // writeln!(stdout, "SPI sr: {:04X}", spi.spi.sr.read().bits()).unwrap();
        led_red.set_low();
        led_green.set_high();
        send_colors(&mut spi, &colors)
            .unwrap_or_else(|_e| {
                led_red.set_high();
                // writeln!(stdout, "SPI send: {:?}", e).unwrap();
            });
        led_green.set_low();

        led_blue.set_high();
        // 500us + room for transmission
        delay.delay_us(500u16);
        led_blue.set_low();
    }
}
