#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use defmt_rtt as _;

#[rtic::app(device = rp_pico::hal::pac, peripherals = true, dispatchers = [TIMER_IRQ_1])]
mod app {

    use defmt::*;
    use embedded_hal::digital::v2::OutputPin;
    // use embedded_hal::blocking::spi;
    // use embedded_time::{duration::*, rate::*};
    use rp2040_monotonic::*;

    #[monotonic(binds = TIMER_IRQ_0, default = true)]
    type MyMono = Rp2040Monotonic;

    use rp_pico::{
        hal::{self, clocks::{init_clocks_and_plls}, watchdog::Watchdog, Sio, Clock},
        XOSC_CRYSTAL_FREQ,
    };

    #[shared]
    struct Shared
    {
        led: hal::gpio::Pin<hal::gpio::pin::bank0::Gpio25, hal::gpio::PushPullOutput>,
        spi: hal::Spi<hal::spi::Enabled, hal::pac::SPI0, 8>
        // spi: hal::Spi<hal::spi::Enabled, D, 8>
    }

    #[local]
    struct Local {}

    #[init]
    fn init(c: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut resets = c.device.RESETS;
        let mut watchdog = Watchdog::new(c.device.WATCHDOG);
        let clocks = init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            c.device.XOSC,
            c.device.CLOCKS,
            c.device.PLL_SYS,
            c.device.PLL_USB,
            &mut resets,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let sio = Sio::new(c.device.SIO);
        let pins = rp_pico::Pins::new(
            c.device.IO_BANK0,
            c.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets,
        );
        let mut led = pins.led.into_push_pull_output();
        led.set_low().unwrap();


        let _lora_sck = pins.gpio18.into_mode::<hal::gpio::FunctionSpi>();
        let _lora_mosi = pins.gpio19.into_mode::<hal::gpio::FunctionSpi>();
        let _lora_miso = pins.gpio16.into_mode::<hal::gpio::FunctionSpi>();
        let _lora_nss = pins.gpio17.into_push_pull_output();
        let _lora_dio0 = pins.gpio21.into_push_pull_output();
        let _lora_reset = pins.gpio20.into_push_pull_output();
        // let spi = hal::spi::Spi::<_, _, 8>::new(c.device.SPI0);
        
        // spi.init(&mut resets, clocks.peripheral_clock.freq(), 16_000_000u32.Hz(), &embedded_hal::spi::MODE_0);
        let spi = hal::Spi::<_, _, 8>::new(c.device.SPI0);
        let spi = spi.init(
            &mut resets,
            clocks.peripheral_clock.freq(),
            embedded_time::rate::Hertz(1_000_000_u32),
            &embedded_hal::spi::MODE_0
        );

        let mono = rp2040_monotonic::Rp2040Monotonic::new(c.device.TIMER);
        foo::spawn_after(1.secs()).unwrap();
        bar::spawn_after(1.secs()).unwrap();

        (Shared { led, spi }, Local {}, init::Monotonics(mono))
    }
    #[task(
        shared = [led],
        local = [tog: bool = true],
    )]
    fn foo(mut c: foo::Context) {
        info!("foo");

        if *c.local.tog {
            c.shared.led.lock(|l| l.set_high().unwrap());
        } else {
            c.shared.led.lock(|l| l.set_low().unwrap());
        }
        *c.local.tog = !*c.local.tog;
        // foo::spawn_after(1.secs()).unwrap();
    }
    #[task(
        shared = [spi],
        local = [],
    )]
    fn bar(mut c: bar::Context) {
        info!("bar");
        c.shared.spi.lock(|s| {
            send_spi(s, 0xaa_u8);
        });
        bar::spawn_after(1.secs()).unwrap();
    }
    fn send_spi<S>(spi: &mut S, word: u8) 
    where S: embedded_hal::spi::FullDuplex<u8> {
        let _ = spi.send(word);
    }
}
