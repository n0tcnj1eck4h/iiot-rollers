mod encoder;
mod motor;

use std::{error::Error, thread, time::Duration};

use encoder::{EncoderPin, handle_encoder_interrupt};
use rppal::gpio::{Gpio, Trigger};

use crate::{
    encoder::{get_encoder_count, reset_encoder_count},
    motor::Motor,
};

const COMMAND_TOPIC: &str = "home/shutter/set/perentage";
const STATUS_TOPIC: &str = "home/shutter/status";

const CLIENT_ID: &str = "rumqtt-async";
const HOST: &str = "test.mosquitto.org";
const PORT: u16 = 1883;

// async fn command_reciever(mut event_loop: EventLoop) {
//     loop {
//         while let Ok(notification) = event_loop.poll().await {
//             println!("Received = {:?}", notification);
//         }
//         println!("Connection lost. Reconnecting...");
//     }
// }

// async fn blink_task() {
//     loop {
//         println!("On...");
//         tx.send(MotorState::Open).await.unwrap();
//         time::sleep(Duration::from_millis(500)).await;
//         println!("Off...");
//         tx.send(MotorState::Stop).await.unwrap();
//         time::sleep(Duration::from_millis(500)).await;
//     }
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new().unwrap();

    let mut motor = Motor {
        a: gpio.get(12)?.into_output_low(),
        b: gpio.get(13)?.into_output_low(),
    };
    motor.stop();

    let mut clk = gpio.get(2)?.into_input();
    let mut dt = gpio.get(3)?.into_input();

    // let mut mqttoptions = MqttOptions::new(CLIENT_ID, HOST, PORT);
    // mqttoptions.set_keep_alive(Duration::from_secs(5));

    // let (client, event_loop) = AsyncClient::new(mqttoptions, 10);
    // client
    //     .subscribe(COMMAND_TOPIC, QoS::AtMostOnce)
    //     .await
    //     .unwrap();

    // let command_task = command_reciever(event_loop);

    // let (tx, rx) = mpsc::channel(32);

    // let percentage = AtomicU8::new(0);

    // tokio::join!(motor_driver_task(rx, close_pin, open_pin),);

    let debounce = None;

    clk.set_async_interrupt(Trigger::Both, debounce, |event| {
        handle_encoder_interrupt(EncoderPin::CLK, event);
    })?;

    dt.set_async_interrupt(Trigger::Both, debounce, |event| {
        handle_encoder_interrupt(EncoderPin::DT, event);
    })?;

    let stdin = std::io::stdin();
    loop {
        // println!("Encoder: {}", get_encoder_count());
        let mut buffer = String::new();
        stdin.read_line(&mut buffer)?;
        let mut split = buffer.as_str().trim().split(char::is_whitespace);
        let command = split.next().unwrap();
        match command {
            "calibrate" => {
                reset_encoder_count();
                motor.open();
                thread::sleep(Duration::from_millis(200));
                motor.stop();
                if get_encoder_count() < 0 {
                    motor.flip_polarity();
                }
            }
            "open" => motor.open(),
            "close" => motor.close(),
            "stop" => motor.stop(),
            "enc" => println!("Encoder: {}", get_encoder_count()),
            "go" => {
                let arg: Option<i32> = split.next().map(str::parse).map(Result::ok).flatten();
                if let Some(desired_pos) = arg {
                    let enc = get_encoder_count();
                    if desired_pos > enc {
                        motor.open();
                        while desired_pos > get_encoder_count() {}
                        motor.stop();
                    } else if desired_pos < enc {
                        motor.close();
                        while desired_pos < get_encoder_count() {}
                        motor.stop();
                    }
                }
            }
            "exit" => break,
            _ => println!("what"),
        }
    }
    println!("Exiting...");
    Ok(())
}
