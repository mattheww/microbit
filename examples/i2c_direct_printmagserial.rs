#![no_main]
#![no_std]

use panic_halt as _;

use microbit::hal::nrf51::*;
use microbit::hal::nrf51::{interrupt, UART0};

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::Peripherals;

use core::cell::RefCell;
use core::fmt::Write;
use cortex_m_rt::entry;

static RTC: Mutex<RefCell<Option<RTC0>>> = Mutex::new(RefCell::new(None));
static UART: Mutex<RefCell<Option<UART0>>> = Mutex::new(RefCell::new(None));
static TWI: Mutex<RefCell<Option<TWI1>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let Some(p) = microbit::Peripherals::take() {
        p.CLOCK.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });

        while p.CLOCK.events_lfclkstarted.read().bits() == 0 {}

        /* And then set it back to 0 again, just because ?!? */
        p.CLOCK.events_lfclkstarted.write(|w| unsafe { w.bits(0) });

        p.GPIO.pin_cnf[24].write(|w| w.pull().pullup().dir().output());
        p.GPIO.pin_cnf[25].write(|w| w.pull().disabled().dir().input());

        p.UART0.pseltxd.write(|w| unsafe { w.bits(24) });
        p.UART0.pselrxd.write(|w| unsafe { w.bits(25) });

        p.UART0.baudrate.write(|w| w.baudrate().baud115200());
        p.UART0.enable.write(|w| w.enable().enabled());

        let _ = write!(
            UART0Buffer(&p.UART0),
            "\n\rWelcome to the magnetometer reader!\n\r"
        );

        p.RTC0.prescaler.write(|w| unsafe { w.bits(4095) });
        p.RTC0.evtenset.write(|w| w.tick().set_bit());
        p.RTC0.intenset.write(|w| w.tick().set_bit());
        p.RTC0.tasks_start.write(|w| unsafe { w.bits(1) });

        /* Prepare PIN0 and PIN30 for I2C SDA and SCL */
        p.GPIO.pin_cnf[0].write(|w| w.pull().disabled().dir().input().drive().s0d1());
        p.GPIO.pin_cnf[30].write(|w| w.pull().disabled().dir().input().drive().s0d1());

        {
            let twi = &p.TWI1;

            /* Set pins 0 and 30 for I2C SDA and SCL */
            twi.pselscl.write(|w| unsafe { w.bits(0) });
            twi.pselsda.write(|w| unsafe { w.bits(30) });

            /* Enable I2C */
            twi.enable.write(|w| w.enable().enabled());

            /* Configure magnetometer for automatic updates */
            twi.address.write(|w| unsafe { w.address().bits(0x0E) });
            twi.tasks_starttx.write(|w| unsafe { w.bits(1) });

            twi.txd.write(|w| unsafe { w.bits(0x10) });
            while twi.events_txdsent.read().bits() == 0 {}
            twi.events_txdsent.write(|w| unsafe { w.bits(0) });

            twi.txd.write(|w| unsafe { w.bits(0x1) });
            while twi.events_txdsent.read().bits() == 0 {}
            twi.events_txdsent.write(|w| unsafe { w.bits(0) });
            twi.tasks_stop.write(|w| unsafe { w.bits(1) });

            twi.address.write(|w| unsafe { w.address().bits(0x0E) });
            twi.tasks_starttx.write(|w| unsafe { w.bits(1) });

            twi.txd.write(|w| unsafe { w.bits(0x11) });
            while twi.events_txdsent.read().bits() == 0 {}
            twi.events_txdsent.write(|w| unsafe { w.bits(127) });

            twi.txd.write(|w| unsafe { w.bits(0x1) });
            while twi.events_txdsent.read().bits() == 0 {}
            twi.events_txdsent.write(|w| unsafe { w.bits(0) });
            twi.tasks_stop.write(|w| unsafe { w.bits(1) });
        }

        cortex_m::interrupt::free(move |cs| {
            *RTC.borrow(cs).borrow_mut() = Some(p.RTC0);
            *UART.borrow(cs).borrow_mut() = Some(p.UART0);
            *TWI.borrow(cs).borrow_mut() = Some(p.TWI1);
        });

        if let Some(mut p) = Peripherals::take() {
            p.NVIC.enable(microbit::Interrupt::RTC0);
            microbit::NVIC::unpend(microbit::Interrupt::RTC0);
        }
    }

    loop {
        continue;
    }
}

// Define an interrupt handler, i.e. function to call when exception occurs. Here if our timer
// trips, we'll read data from the accelerometer
#[interrupt]
fn RTC0() {
    /* Enter critical section */
    cortex_m::interrupt::free(|cs| {
        if let (Some(rtc), Some(twi)) = (
            RTC.borrow(cs).borrow().as_ref(),
            TWI.borrow(cs).borrow().as_ref(),
        ) {
            let mut data: [u8; 6] = [0; 6];

            /* Request data */
            twi.address.write(|w| unsafe { w.address().bits(0x0E) });
            twi.tasks_starttx.write(|w| unsafe { w.bits(1) });
            twi.txd.write(|w| unsafe { w.bits(0x1) });
            while twi.events_txdsent.read().bits() == 0 {}
            twi.events_txdsent.write(|w| unsafe { w.bits(0) });

            /* Turn around to read data */
            twi.shorts.write(|w| w.bb_suspend().enabled());
            twi.tasks_startrx.write(|w| unsafe { w.bits(1) });

            /* Get 5 values */
            for d in &mut data {
                while twi.events_rxdready.read().bits() == 0 {}
                *d = twi.rxd.read().bits() as u8;
                twi.events_rxdready.write(|w| unsafe { w.bits(0) });

                twi.tasks_resume.write(|w| unsafe { w.bits(1) });
            }

            /* Get the last value */
            twi.shorts.write(|w| w.bb_stop().enabled());
            twi.tasks_resume.write(|w| unsafe { w.bits(1) });

            while twi.events_rxdready.read().bits() == 0 {}
            data[5] = twi.rxd.read().bits() as u8;
            twi.events_rxdready.write(|w| unsafe { w.bits(0) });

            /* Join and translate 2s complement values */

            let (x, y, z) = (
                (u16::from(data[0]) << 8 | u16::from(data[1])) as i16,
                (u16::from(data[2]) << 8 | u16::from(data[3])) as i16,
                (u16::from(data[4]) << 8 | u16::from(data[5])) as i16,
            );

            /* Print read values on the serial console */
            if let Some(uart) = UART.borrow(cs).borrow().as_ref() {
                let _ = write!(UART0Buffer(uart), "x: {}, y: {}, z: {}\n\r", x, y, z);
            }

            /* Clear timer event */
            rtc.events_tick.write(|w| unsafe { w.bits(0) });
        }
    });

    loop {
        continue;
    }
}

pub struct UART0Buffer<'a>(pub &'a UART0);

impl<'a> core::fmt::Write for UART0Buffer<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let uart0 = self.0;
        uart0.tasks_starttx.write(|w| unsafe { w.bits(1) });
        for c in s.as_bytes() {
            /* Write the current character to the output register */
            uart0.txd.write(|w| unsafe { w.bits(u32::from(*c)) });

            /* Wait until the UART is clear to send */
            while uart0.events_txdrdy.read().bits() == 0 {}

            /* And then set it back to 0 again, just because ?!? */
            uart0.events_txdrdy.write(|w| unsafe { w.bits(0) });
        }
        uart0.tasks_stoptx.write(|w| unsafe { w.bits(1) });
        Ok(())
    }
}
