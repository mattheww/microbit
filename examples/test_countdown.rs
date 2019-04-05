#![no_main]
#![no_std]

extern crate cortex_m_rt;
extern crate microbit;
extern crate panic_halt;

use core::time::Duration;
use cortex_m_rt::entry;
use microbit::hal::prelude::*;
use microbit::hal::hi_res_timer::TimerFrequency;
use microbit::hal::lo_res_timer::{FREQ_8HZ, FREQ_32768HZ};
use microbit::hal::timer::{CountDownTimer, CountDownRtc};
//use microbit::nrf51::{TIMER0, TIMER1, TIMER2, RTC0, RTC1};
use nb::block;

/* 
@startuml

scale 500 as 60 pixels
skinparam monochrome reverse

robust "LED light" as LED
concise "Timer 2" as TIMER2
concise "Timer 1" as TIMER1
concise "Timer 0" as TIMER0
concise "RTC 1" as RTC1
concise "RTC 0" as RTC0

LED is Off

@0
TIMER2 is 500ms
TIMER1 is 1000ms
TIMER0 is 1500ms
RTC1 is 4000ms
RTC0 is 5000ms

@+500
LED is On
TIMER2 is " "
TIMER2 -> LED

@+500
LED is Off
TIMER2 is " "
TIMER1 is 1000ms
TIMER1 -> LED

@+500
LED is On
TIMER2 is " "
TIMER0 is 1500ms
TIMER0 -> LED

@+500
LED is Off
TIMER2 is 500ms
TIMER1 is " "
TIMER1 -> LED

@+500
LED is On
TIMER2 is " "
TIMER2 -> LED

@+500
LED is Off
TIMER2 is " "
TIMER1 is " "
TIMER0 is " "
TIMER0 -> LED

@4000
LED is On
RTC1 is " "
RTC1 -> LED

@5000
LED is Off
RTC0 is " "
RTC0 -> LED

@enduml
*/

#[entry]
fn main() -> ! {
    if let Some(p) = microbit::Peripherals::take() {

        // Start the LFCLK
        p.CLOCK.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });
        while p.CLOCK.events_lfclkstarted.read().bits() == 0 {}
        p.CLOCK.events_lfclkstarted.reset();

        let gpio = p.GPIO.split();

        let mut pin_mid = gpio.pin14.into_push_pull_output();
        let mut pin_lower_left = gpio.pin15.into_push_pull_output();
        let mut pin_upper_right = gpio.pin13.into_push_pull_output();
        let _ = gpio.pin6.into_push_pull_output();

        // 32bits @ 1MHz = ~72 minutes
        let mut timer0 = CountDownTimer::new(p.TIMER0, TimerFrequency::Freq1MHz);
        // 16bits @ 31.25kHz = ~2 seconds
        let mut timer1 = CountDownTimer::new(p.TIMER1, TimerFrequency::Freq31250Hz);
        // 16bits @ 31.25kHz = ~2 seconds
        let mut timer2 = CountDownTimer::new(p.TIMER2, TimerFrequency::Freq31250Hz);

        // 24bits @ 8Hz = ~24 days
        let mut rtc0 = CountDownRtc::new(p.RTC0, FREQ_8HZ);
        // 24bits @ 32.768kHz = 512 seconds
        let mut rtc1 = CountDownRtc::new(p.RTC1, FREQ_32768HZ);

        rtc0.start(Duration::from_millis(5_000));
        rtc0.start(Duration::from_millis(5_000));
        rtc1.start(Duration::from_millis(4_000));

        timer0.start(Duration::from_millis(1_500));
        timer1.start(Duration::from_millis(1_000));
        timer2.start(Duration::from_millis(500));

        // @+500
        block!(timer2.wait()).unwrap();
        pin_mid.set_high();

        // @+500
        block!(timer1.wait()).unwrap();
        pin_mid.set_low();

        // @+500
        block!(timer0.wait()).unwrap();
        pin_lower_left.set_high();

        // @+500
        block!(timer1.wait()).unwrap();
        pin_lower_left.set_low();
        timer2.start(Duration::from_millis(500));

        // @+500
        block!(timer2.wait()).unwrap();
        pin_upper_right.set_high();

        // @+500
        block!(timer0.wait()).unwrap();
        pin_upper_right.set_low();

        // @4000
        block!(rtc1.wait()).unwrap();
        pin_mid.set_high();

        // @5000
        block!(rtc0.wait()).unwrap();
        pin_mid.set_low();


        // Test reusing an RTC
        rtc0.start(Duration::from_millis(1000));
        block!(rtc0.wait()).unwrap();
        pin_mid.set_high();

        rtc0.start(Duration::from_millis(1000));
        block!(rtc0.wait()).unwrap();
        pin_mid.set_low();

    }

    panic!("FIN");
}
