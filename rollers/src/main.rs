mod encoder;
mod motor;

use std::error::Error;

use encoder::{EncoderPin, handle_encoder_interrupt};
use rppal::gpio::{Gpio, Trigger};

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

    // let close_pin = gpio.get(6).unwrap().into_output_low();
    // let open_pin = gpio.get(5).unwrap().into_output_low();

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

    loop {}
    println!("Exiting...");
    Ok(())
}
