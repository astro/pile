//! ## TODO
//!
//! * Watchdog
//! * mDNS
#![no_main]
#![no_std]
#![feature(used)]

extern crate cortex_m;
#[macro_use(entry, exception)]
extern crate cortex_m_rt;
extern crate panic_itm;
extern crate cortex_m_semihosting;
#[macro_use(interrupt)]
extern crate stm32f429;
extern crate stm32f429_hal;
extern crate embedded_hal;
extern crate stm32_eth as eth;
extern crate smoltcp;
extern crate managed;
#[macro_use(block)]
extern crate nb;

use core::slice;
use core::cell::RefCell;
use core::ops::DerefMut;
use cortex_m::interrupt::Mutex;
use cortex_m::asm;
use cortex_m_rt::ExceptionFrame;
use stm32f429::{Interrupt, Peripherals, CorePeripherals, SCB, DEVICE_ID, TIM2};
use stm32f429_hal::flash::FlashExt;
use stm32f429_hal::rcc::RccExt;
use stm32f429_hal::gpio::GpioExt;
use stm32f429_hal::time::U32Ext;
use stm32f429_hal::delay::Delay;
use stm32f429_hal::timer::{Timer, Event};
use stm32f429_hal::spi::Spi;
use stm32f429_hal::dma::{DmaExt, Transfer};
use stm32f429_hal::watchdog::{IndependentWatchdog, Watchdog};
use embedded_hal::digital::OutputPin;
use embedded_hal::spi;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::timer::CountDown;

use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, IpEndpoint,
                    Ipv4Address};
use smoltcp::iface::{NeighborCache, EthernetInterfaceBuilder, Routes};
use smoltcp::dhcp::Dhcpv4Client;
use smoltcp::socket::{SocketSet, UdpPacketMetadata, RawSocketBuffer, RawPacketMetadata};
use eth::{Eth, RingEntry};

mod udp_proto;
use udp_proto::Receiver;

mod mac_gen;
use mac_gen::MacAddrGenerator;

// mod spi_dev;
mod ws2812_spi;
use ws2812_spi::TimedData;

// use core::fmt::Write;
// use cortex_m_semihosting::hio;

static TIME: Mutex<RefCell<u64>> = Mutex::new(RefCell::new(0));
static ETH_PENDING: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));
static TIMER: Mutex<RefCell<Option<Timer<TIM2>>>> = Mutex::new(RefCell::new(None));
const TIMER_RATE: u32 = 4;

pub fn now() -> u64 {
    cortex_m::interrupt::free(|cs| {
        *TIME.borrow(cs)
            .borrow()
    })
}


entry!(entry);
fn entry() -> ! {
    loop {
        main();
    }
}

fn main() {
    // let mut stdout = hio::hstdout().unwrap();

    // Board setup
    let p = Peripherals::take().unwrap();
    let mut cp = CorePeripherals::take().unwrap();

    let mut wdog = IndependentWatchdog::new(p.IWDG, 3_000);

    let mut scb = cp.SCB;
    if ! SCB::icache_enabled() {
        scb.enable_icache();
    }
    if ! SCB::dcache_enabled() {
        let cpuid = &mut cp.CPUID;
        scb.enable_dcache(cpuid);
    }
    
    let mut rcc = p.RCC.constrain();
    
    let mut gpiob = p.GPIOB.split(&mut rcc.ahb1);
    let mut gpioc = p.GPIOC.split(&mut rcc.ahb1);
    let mut gpiod = p.GPIOD.split(&mut rcc.ahb1);
    let mut led_green = gpiob.pb0.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let mut led_blue = gpiob.pb7.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let mut led_red = gpiob.pb14.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    led_red.set_high();
    
    // clock configuration
    let mut flash = p.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut tim2 = Timer::tim2(p.TIM2, TIMER_RATE.hz(), clocks, &mut rcc.apb1);
    tim2.listen(Event::TimeOut);
    cortex_m::interrupt::free(|cs| {
        *TIMER.borrow(cs).borrow_mut() = Some(tim2);
    });
    cp.NVIC.enable(Interrupt::TIM2);

    // WS2801/WS2812 setup
    let mosi = gpiob.pb15.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
    let miso = gpioc.pc2.into_af5(&mut gpioc.moder, &mut gpioc.afrl);
    let sck = gpiod.pd3.into_af5(&mut gpiod.moder, &mut gpiod.afrl);
    // let _nss = gpiob.pb12.into_af5(&mut gpiob.moder, &mut gpiob.afrh);

    let spi_mode = spi::Mode {
        polarity: spi::Polarity::IdleLow,
        phase: spi::Phase::CaptureOnFirstTransition,
    };
    let mut spi = Spi::spi2(p.SPI2, (sck, miso, mosi), spi_mode, (ws2812_spi::RESAMPLED_KHZ as u32).khz(), clocks, &mut rcc.apb1);

    let streams = p.DMA1.split(&mut rcc.ahb1);
    let mut spi_dma = Some(streams.s4);

    led_red.set_low();

    // Ethernet setup
    unsafe { eth::setup(&Peripherals::steal()); }
    let mut rx_ring: [RingEntry<_>; 4] = Default::default();
    let mut tx_ring: [RingEntry<_>; 2] = Default::default();
    let mut eth = Eth::new(
        p.ETHERNET_MAC, p.ETHERNET_DMA,
        &mut rx_ring[..], &mut tx_ring[..]
    );
    eth.enable_interrupt(&mut cp.NVIC);

    led_red.set_low();

    let mut ip_addrs = [IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0)];
    let mut neighbor_storage = [None; 16];
    let neighbor_cache = NeighborCache::new(&mut neighbor_storage[..]);
    let mut mac_gen = MacAddrGenerator::new();
    let udid = unsafe { slice::from_raw_parts(
        DEVICE_ID::ptr() as *const u8,
        12)
    };
    // writeln!(stdout, "UDID: {:?}", udid);
    mac_gen.feed(udid.iter().cloned());
    let mac_addr = mac_gen.into_addr();
    // writeln!(stdout, "MAC: {:?}", mac_addr);
    let ethernet_addr = EthernetAddress(mac_addr);
    let mut routes_storage = [None; 1];
    let routes = Routes::new(&mut routes_storage[..]);
    let mut iface = EthernetInterfaceBuilder::new(&mut eth)
        .ethernet_addr(ethernet_addr)
        .ip_addrs(&mut ip_addrs[..])
        .neighbor_cache(neighbor_cache)
        .routes(routes)
        .finalize();

    let mut udp_tx_buffer_m = [UdpPacketMetadata::EMPTY; 1];
    let mut udp_tx_buffer = [0u8; 128];
    let mut udp_rx_buffer_m = [UdpPacketMetadata::EMPTY; 4];
    let mut udp_rx_buffer = [0u8; 8192];

    let mut dhcp_rx_buffer_m = [RawPacketMetadata::EMPTY; 2];
    let mut dhcp_rx_buffer = [0u8; 3000];
    let dhcp_rx = RawSocketBuffer::new(&mut dhcp_rx_buffer_m[..], &mut dhcp_rx_buffer[..]);
    let mut dhcp_tx_buffer_m = [RawPacketMetadata::EMPTY; 2];
    let mut dhcp_tx_buffer = [0u8; 3000];
    let dhcp_tx = RawSocketBuffer::new(&mut dhcp_tx_buffer_m[..], &mut dhcp_tx_buffer[..]);

    let mut sockets_storage = [None, None, None];
    let mut sockets = SocketSet::new(&mut sockets_storage[..]);
    let mut dhcp = Dhcpv4Client::new(&mut sockets, dhcp_rx, dhcp_tx, Instant::from_millis(0));

    let mut udp_receiver = Receiver::new(
        IpEndpoint::new(IpAddress::default(), 2342),
        &mut sockets,
        (&mut udp_rx_buffer_m[..], &mut udp_rx_buffer[..]),
        (&mut udp_tx_buffer_m[..], &mut udp_tx_buffer[..])
    );

    // Init animation
    // writeln!(stdout, "INIT");
    let mut output_buffer = [0u8; ws2812_spi::SAMPLERATE * 4 * 640];
    let mut init_colors = [0u8; 4 * 640];
    let init_colors_len = init_colors.len() / 4;
    for len in 1..init_colors_len {
        // writeln!(stdout, "i {}", len);
        for (i, color) in init_colors.iter_mut().enumerate() {
            let i = i as u8;
            match i % 4 {
                0 => *color = i / 4,
                1 => *color = 255 - (i / 4),
                2 => *color = len as u8,
                3 => *color = 31 * ((i / 4) % 8),
                _ => unreachable!()
            }
        }
        led_blue.set_high();
        let data = TimedData::encode(
            &init_colors[0..(4 * len)],
            &mut output_buffer[..]
        );
        spi_dma = spi.dma_write(
            spi_dma.take().unwrap(),
            &data.as_ref()[..ws2812_spi::SAMPLERATE * 4 * len]
        ).wait()
            .map(|spi_dma| {
                delay.delay_us(500u16);
                Some(spi_dma)
            })
            .unwrap_or_else(|spi_dma| Some(spi_dma));
        led_blue.set_low();

        wdog.reload();
    }
    // Red
    for (i, color) in init_colors.iter_mut().enumerate() {
        match i % 4 {
            0 => *color = 1,
            _ => *color = 0,
        }
    }
    led_blue.set_high();
    spi_dma = spi.dma_write(
        spi_dma.take().unwrap(),
        &init_colors[..]
    ).wait()
        .map(|spi_dma| {
            delay.delay_us(500u16);
            Some(spi_dma)
        })
        .unwrap_or_else(|spi_dma| Some(spi_dma));
    led_blue.set_low();

    // Main loop
    // writeln!(stdout, "loop");
    loop {
        cortex_m::interrupt::free(|cs| {
            let mut eth_pending =
                ETH_PENDING.borrow(cs)
                .borrow_mut();
            *eth_pending = false;
        });
        led_red.set_low();

        let now = Instant::from_millis(now() as i64);
        // writeln!(stdout, "Poll {}", now);
        let eth_sent = iface.poll(&mut sockets, now)
            .unwrap_or_else(|_| {
                led_red.set_high();
                true
            });
        dhcp.poll(&mut iface, &mut sockets, now)
            .map(|_| ())
            // .unwrap_or_else(|e| writeln!(stdout, "DHCP: {:?}", e).unwrap());
            .unwrap_or(());
        udp_receiver.poll(&mut sockets, now, |pixels| {
            // let sum = pixels.iter()
            //     .fold(0, |sum, i| sum + i);
            // writeln!(stdout, "Received {} bytes, sum: {:02X}", pixels.len(), sum).unwrap();
            // for c in pixels {
            //     write!(stdout, "{:02X} ", c);
            // }
            // writeln!(stdout, "");
            led_blue.set_high();
            // leds.write(pixels)
            //     .unwrap_or_else(|e| {
            //         writeln!(stdout, "recv: {:?}", e).unwrap();
            //         led_red.set_high()
            //     });
            spi_dma = spi.dma_write(
                spi_dma.take().unwrap(),
                pixels
            ).wait()
                .map(|spi_dma| {
                    delay.delay_us(500u16);
                    Some(spi_dma)
                })
                .unwrap_or_else(|spi_dma| Some(spi_dma));
            led_blue.set_low();
        })
            .unwrap_or_else(|_| led_red.set_high());

        if ! eth_sent {
            // Sleep if no ethernet work is pending
            cortex_m::interrupt::free(|cs| {
                let eth_pending =
                    ETH_PENDING.borrow(cs)
                    .borrow_mut();
                if ! *eth_pending {
                    led_green.set_high();
                    asm::wfi();
                    // Awaken by interrupt
                    led_green.set_low();
                }
            });
        }

        wdog.reload();
    }
}

exception!(*, default_handler);
fn default_handler(_intr: i16) {
}

exception!(HardFault, fault_handler);
fn fault_handler(_: &ExceptionFrame) -> ! {
    // asm::bkpt();
    loop {}
}

fn eth_interrupt_handler() {
    let p = unsafe { Peripherals::steal() };

    cortex_m::interrupt::free(|cs| {
        let mut eth_pending =
            ETH_PENDING.borrow(cs)
            .borrow_mut();
        *eth_pending = true;
    });

    // Clear interrupt flags
    eth::eth_interrupt_handler(&p.ETHERNET_DMA);
}

#[used]
interrupt!(ETH, eth_interrupt_handler);

fn tim2_interrupt_handler() {
    cortex_m::interrupt::free(|cs| {
        match TIMER.borrow(cs).borrow_mut().deref_mut() {
            &mut Some(ref mut timer) => {
                // Clear update interrupt flag
                block!(timer.wait());
            }
            _ => {}
        }

        let mut time =
            TIME.borrow(cs)
            .borrow_mut();
        *time += u64::from(1000u32 / TIMER_RATE);
    });
}

#[used]
interrupt!(TIM2, tim2_interrupt_handler);
