use std::time::Duration;

use rppal::gpio::OutputPin;
use tokio::{sync::mpsc, time};

pub struct Motor {
    // state: MotorState,
    pub a: OutputPin,
    pub b: OutputPin,
}

impl Motor {
    pub fn open(&mut self) {
        println!("Opening...");
        self.a.set_low();
        self.b.set_high();
    }

    pub fn close(&mut self) {
        println!("Closing...");
        self.b.set_low();
        self.a.set_high();
    }

    pub fn stop(&mut self) {
        println!("Stopping...");
        self.a.set_low();
        self.b.set_low();
    }

    pub fn flip_polarity(&mut self) {
        std::mem::swap(&mut self.a, &mut self.b);
    }
}

pub enum MotorState {
    Stop,
    Close,
    Open,
}

// pub async fn motor_driver_task(
//     mut rv: mpsc::Receiver<MotorState>,
//     mut a: OutputPin,
//     mut b: OutputPin,
//     mut c: OutputPin,
//     mut d: OutputPin,
// ) {
//     while let Some(state) = rv.recv().await {
//         match state {
//             MotorState::Stop => {
//                 close_pin.set_low();
//                 open_pin.set_low();
//             }
//             MotorState::Close => {
//                 open_pin.set_low();
//                 time::sleep(Duration::from_millis(10)).await;
//                 close_pin.set_high();
//             }
//             MotorState::Open => {
//                 close_pin.set_low();
//                 time::sleep(Duration::from_millis(10)).await;
//                 open_pin.set_high();
//             }
//         }
//     }
// }
