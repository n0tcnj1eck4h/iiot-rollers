use std::time::Duration;

use rppal::gpio::OutputPin;
use tokio::{sync::mpsc, time};

pub(crate) enum MotorState {
    Stop,
    Close,
    Open,
}

pub(crate) async fn motor_driver_task(
    mut rv: mpsc::Receiver<MotorState>,
    mut close_pin: OutputPin,
    mut open_pin: OutputPin,
) {
    while let Some(state) = rv.recv().await {
        match state {
            MotorState::Stop => {
                close_pin.set_low();
                open_pin.set_low();
            }
            MotorState::Close => {
                open_pin.set_low();
                time::sleep(Duration::from_millis(10)).await;
                close_pin.set_high();
            }
            MotorState::Open => {
                close_pin.set_low();
                time::sleep(Duration::from_millis(10)).await;
                open_pin.set_high();
            }
        }
    }
}
