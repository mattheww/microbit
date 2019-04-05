#![no_main]
#![no_std]

extern crate cortex_m_rt;
extern crate microbit;
extern crate panic_halt;

use cortex_m_rt::entry;
use microbit::hal::prelude::*;
use microbit::hal::lo_res_timer::{FREQ_1024HZ, FREQ_32768HZ};
use microbit::hal::hi_res_timer::TimerFrequency;
use microbit::hal::delay::{DelayTimer, DelayRtc};

#[entry]
fn main() -> ! {
    if let Some(p) = microbit::Peripherals::take() {

        // Start the LFCLK
        p.CLOCK.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });
        while p.CLOCK.events_lfclkstarted.read().bits() == 0 {}
        p.CLOCK.events_lfclkstarted.reset();

        let gpio = p.GPIO.split();

        let mut pin = gpio.pin14.into_push_pull_output();
        let _ = gpio.pin6.into_push_pull_output();

        // 32bits @ 1MHz = ~72 minutes
        let mut delay_timer0 = DelayTimer::new(p.TIMER0, TimerFrequency::Freq1MHz);
        // 16bits @ 31.25kHz = ~2 seconds
        let mut delay_timer1 = DelayTimer::new(p.TIMER1, TimerFrequency::Freq31250Hz);
        // 16bits @ 31.25kHz = ~2 seconds
        let mut delay_timer2 = DelayTimer::new(p.TIMER2, TimerFrequency::Freq31250Hz);

        // 24bits @ 1024Hz = ~4.5 hours
        let mut delay_rtc3 = DelayRtc::new(p.RTC0, FREQ_1024HZ);
        // 24bits @ 32.768kHz = 512 seconds
        let mut delay_rtc4 = DelayRtc::new(p.RTC1, FREQ_32768HZ);

        const LONG_MS: u16 = 800;
        const SHORT_MS: u16 = 400;
        const LONG_US: u32 = 800_000;
        const SHORT_US: u32 = 400_000;

        for _ in 0..2 {
            pin.set_high();
            delay_timer0.delay_ms(LONG_MS);
            pin.set_low();
            delay_timer0.delay_ms(SHORT_MS);
        }

        for _ in 0..2 {
            pin.set_high();
            delay_timer1.delay_ms(LONG_MS);
            pin.set_low();
            delay_timer1.delay_ms(SHORT_MS);
        }

        for _ in 0..2 {
            pin.set_high();
            delay_timer2.delay_ms(LONG_MS);
            pin.set_low();
            delay_timer2.delay_ms(SHORT_MS);
        }

        for _ in 0..2 {
            pin.set_high();
            delay_rtc3.delay_ms(LONG_MS);
            pin.set_low();
            delay_rtc3.delay_ms(SHORT_MS);
        }

        for _ in 0..2 {
            pin.set_high();
            delay_rtc4.delay_ms(LONG_MS);
            pin.set_low();
            delay_rtc4.delay_ms(SHORT_MS);
        }

        for _ in 0..2 {
            pin.set_high();
            delay_timer0.delay_us(LONG_US);
            pin.set_low();
            delay_timer0.delay_us(SHORT_US);
        }

        for _ in 0..2 {
            pin.set_high();
            delay_timer1.delay_us(LONG_US);
            pin.set_low();
            delay_timer1.delay_us(SHORT_US);
        }

        for _ in 0..2 {
            pin.set_high();
            delay_timer2.delay_us(LONG_US);
            pin.set_low();
            delay_timer2.delay_us(SHORT_US);
        }

        for _ in 0..2 {
            pin.set_high();
            delay_rtc3.delay_us(LONG_US);
            pin.set_low();
            delay_rtc3.delay_us(SHORT_US);
        }

        for _ in 0..2 {
            pin.set_high();
            delay_rtc4.delay_us(LONG_US);
            pin.set_low();
            delay_rtc4.delay_us(SHORT_US);
        }
    }

    panic!("FIN");
}
