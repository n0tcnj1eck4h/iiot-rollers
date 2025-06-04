use std::{
    error::Error,
    sync::atomic::{AtomicBool, AtomicI32, Ordering},
    time::Duration,
};

use rppal::gpio::{Event, InputPin, Trigger};

#[derive(Debug)]
pub enum EncoderEvent {
    Up,
    Down,
}

#[derive(Debug)]
pub enum EncoderPin {
    CLK,
    DT,
}

static COUNTER: AtomicI32 = AtomicI32::new(0);
static CLK_STATE: AtomicBool = AtomicBool::new(false);
static DT_STATE: AtomicBool = AtomicBool::new(false);

pub fn reset_encoder_count() {
    COUNTER.store(0, Ordering::Relaxed);
}

pub fn get_encoder_count() -> i32 {
    COUNTER.load(Ordering::Relaxed)
}

pub fn handle_encoder_interrupt(pin: EncoderPin, event: Event) {
    use EncoderPin::*;
    use Trigger::FallingEdge;
    use Trigger::RisingEdge;
    let (new_clk, new_dt, increment) = match (
        pin,
        event.trigger,
        CLK_STATE.load(Ordering::Relaxed),
        DT_STATE.load(Ordering::Relaxed),
    ) {
        (_, Trigger::Disabled | Trigger::Both, _, _) => unreachable!(),
        (CLK, RisingEdge, false, false) => (true, false, 1),
        (DT, RisingEdge, true, false) => (true, true, 1),
        (CLK, FallingEdge, true, true) => (false, true, 1),
        (DT, FallingEdge, false, true) => (false, false, 1),

        (DT, RisingEdge, false, false) => (false, true, -1),
        (CLK, RisingEdge, false, true) => (true, true, -1),
        (DT, FallingEdge, true, true) => (true, false, -1),
        (CLK, FallingEdge, true, false) => (false, false, -1),
        (_, _, _clk, _dt) => (_clk, _dt, 0),
    };

    CLK_STATE.store(new_clk, Ordering::Relaxed);
    DT_STATE.store(new_dt, Ordering::Relaxed);

    COUNTER.fetch_add(increment, Ordering::Relaxed);
}
