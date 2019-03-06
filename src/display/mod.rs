//! Support for the 5×5 LED display.
//!
//! # Scope
//!
//! Together with `tiny-led-matrix`, this module provides:
//! - support for driving the LED display from a timer interrupt
//! - ten levels of brightness for each LED
//! - simple 5×5 greyscale and black-and-white image types.
//!
//! The module doesn't define interrupt handlers directly; instead it provides
//! a function to be called from a timer interrupt. It knows how to program
//! one of the micro:bit's timers to provide that interrupt.
//!
//! # Coordinate system
//!
//! The LEDs are identified using (x,y) coordinates as follows:
//!
//! ```text
//! (0,0) ... (4,0)
//!  ...  ...  ...
//! (4,0) ... (4,4)
//! ```
//!
//! # Greyscale model
//!
//! LED brightness levels are described using a scale from 0 (off) to 9
//! (brightest) inclusive.
//!
//! These are converted to time slices using the same timings as used by the
//! [micro:bit MicroPython port][micropython] (this is different to the 0 to
//! 255 scale used by the [micro:bit runtime][dal]).
//!
//! The time slice for each level above 1 is approximately 1.9× the slice for
//! the previous level.
//!
//! An LED with brightness 9 is lit for one third of the time (because
//! internally there are three 'rows' of LEDs which have to be addressed one
//! at a time).
//!
//! # Images and Render
//!
//! The [`Render`] trait defines the interface that an image-like type needs
//! to provide in order to be displayed.
//!
//! It contains a single function: [`brightness_at(x,
//! y)`][display::Render::brightness_at], returning a brightness level.
//!
//! The [`image`] submodule provides two static image types implementing
//! `Render`:
//! - [`GreyscaleImage`], allowing all 9 levels (using one byte for each LED)
//! - [`BitImage`], allowing only 'on' and 'off' (using five bytes)
//!
//! # Display
//!
//! A [`Display`] instance controls the LEDs and programs a timer. There
//! should normally be a single `Display` instance in the program.
//!
//! # Frames
//!
//! Types implementing [`Render`] aren't used directly with the [`Display`];
//! instead they're used to update a [`MicrobitFrame`] instance which is in
//! turn passed to the `Display`.
//!
//! A `MicrobitFrame` instance is a 'compiled' representation of a 5×5
//! greyscale image, in a form that's more directly usable by the display
//! code.
//!
//! This is exposed in the public API so that you can construct the
//! `MicrobitFrame` representation in code running at a low priority. Then
//! only [`Display::set_frame()`] has to be called in code that can't be
//! interrupted by the display timer.
//!
//! # Timer integration
//!
//! The `Display` expects to control a single timer. It can use the
//! micro:bit's `TIMER0`, `TIMER1`, or `TIMER2`.
//!
//! This uses a 6ms period to light each of the three internal LED rows, so
//! that the entire display is updated every 18ms.
//!
//! When rendering greyscale images, the `Display` requests extra interrupts
//! within each 6ms period. It only requests interrupts for the greyscale
//! levels which are actually required for what's currently being displayed.
//!
//! ## Technical details
//!
//! The timer is set to 16-bit mode, using a 62.5kHz clock (16 µs ticks). It
//! resets every 375 ticks.
//!
//! The display code uses the CC0 and CC1 capture/compare registers. You are
//! free to use CC2 and CC3.
//!
//! # Usage
//!
//! Choose a timer to drive the display from (`TIMER0`, `TIMER1`, or
//! `TIMER2`).
//!
//! When your program starts:
//! * call [`initialise_display()`], passing it the timer you chose and and
//! the gpio peripheral
//! * create a [`Display`] struct (a `Display<MicrobitFrame>`).
//!
//! In an interrupt handler for the same timer, call
//! [`handle_display_event()`].
//!
//! To change what's displayed: create a [`MicrobitFrame`] instance, use
//! [`.set()`](`display::Frame::set()`) to put an image (something implementing
//! [`Render`]) in it, then call [`Display::set_frame()`]. Note you'll have to
//! `use microbit::display::Frame` to make `set()` available.
//!
//! You can call `set_frame()` at any time, so long as you're not
//! interrupting, or interruptable by, `handle_display_event()`.
//!
//! Once you've called `set_frame()`, you are free to reuse the
//! `MicrobitFrame`.
//!
//! [dal]: https://lancaster-university.github.io/microbit-docs/
//! [micropython]: https://microbit-micropython.readthedocs.io/
//!
//! [`BitImage`]: display::image::BitImage
//! [`Display`]: display::Display
//! [`Display::set_frame()`]: display::Display::set_frame
//! [`Frame`]: display::Frame
//! [`Matrix`]: display::Matrix
//! [`MicrobitFrame`]: display::MicrobitFrame
//! [`Render`]: display::Render
//! [`image`]: display::image
//! [`handle_display_event()`]: display::handle_display_event
//! [`initialise_display()`]: display::initialise_display
//! [`DisplayTimer`]: tiny_led_matrix::DisplayTimer
//! [`GreyscaleImage`]: display::image::GreyscaleImage
//!

#[doc(no_inline)]
pub use tiny_led_matrix::{
    Render,
    MAX_BRIGHTNESS,
    Display,
    Frame,
};

mod microbit_control;
mod microbit_matrix;
mod microbit_timer;

pub mod image;

pub use microbit_matrix::MicrobitFrame;


use microbit_control::MicrobitGpio;

/// Initialises the micro:bit hardware to use the display driver.
///
/// You can use `nrf51::TIMER0`, `nrf51::TIMER1`, or `nrf51::TIMER2` for the
/// timer parameter.
///
/// Assumes the timer and the GPIO port are in the state they would have after
/// system reset.
///
/// # Example
///
/// ```ignore
/// let mut p: nrf51::Peripherals = _;
/// microbit::display::initialise_display(&mut p.TIMER1, &mut p.GPIO);
/// ```
pub fn initialise_display<'a, T>(
    timer: &'a mut T,
    gpio: &mut crate::hal::nrf51::GPIO)
    where T: microbit_timer::Nrf51Timer<'a> {
    tiny_led_matrix::initialise_control(&mut MicrobitGpio(gpio));
    tiny_led_matrix::initialise_timer(&mut timer.as_display_timer());
}

/// Updates the LEDs and timer state during a timer interrupt.
///
/// You can use `nrf51::TIMER0`, `nrf51::TIMER1`, or `nrf51::TIMER2` for the
/// timer parameter (it must be the same timer you used for
/// [`initialise_display()`]).
///
/// Call this in an interrupt handler for the timer you're using.
///
/// Takes care of clearing the timer's event registers.
///
/// See [`Display::handle_event()`] for details.
///
/// # Example
///
/// In the style of `cortex-m-rtfm` v0.4:
///
/// ```ignore
/// #[interrupt(priority = 2, resources = [TIMER1, GPIO, DISPLAY])]
/// fn TIMER1() {
///     microbit::display::handle_display_event(
///         &mut resources.DISPLAY,
///         resources.TIMER1,
///         resources.GPIO,
///     );
/// }
/// ```
pub fn handle_display_event<'a, T>(
    display: &mut Display<MicrobitFrame>,
    timer: &'a mut T,
    gpio: &mut crate::hal::nrf51::GPIO)
    where T: microbit_timer::Nrf51Timer<'a> {
    display.handle_event(
        &mut timer.as_display_timer(),
        &mut MicrobitGpio(gpio),
    );
}

