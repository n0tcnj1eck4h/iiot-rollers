use rppal::gpio::{Event, Gpio, OutputPin};
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS, RecvError};
use std::time::Duration;
use std::{error::Error, sync::atomic::AtomicBool};
use tokio::{sync::mpsc, task, time};

const COMMAND_TOPIC: &str = "home/shutter/set/perentage";
const STATUS_TOPIC: &str = "home/shutter/status";

const CLIENT_ID: &str = "rumqtt-async";
const HOST: &str = "test.mosquitto.org";
const PORT: u16 = 1883;

async fn command_reciever(mut event_loop: EventLoop) {
    loop {
        while let Ok(notification) = event_loop.poll().await {
            println!("Received = {:?}", notification);
        }
        println!("Connection lost. Reconnecting...");
    }
}

enum MotorState {
    Stop,
    Close,
    Open,
}

async fn motor_driver(
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

#[tokio::main]
async fn main() {
    let gpio = Gpio::new().unwrap();

    let close_pin_id = 6;
    let open_pin_id = 5;

    let close_pin = gpio.get(close_pin_id).unwrap().into_output_low();
    let open_pin = gpio.get(open_pin_id).unwrap().into_output_low();

    let mut mqttoptions = MqttOptions::new(CLIENT_ID, HOST, PORT);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, event_loop) = AsyncClient::new(mqttoptions, 10);
    client
        .subscribe(COMMAND_TOPIC, QoS::AtMostOnce)
        .await
        .unwrap();

    task::spawn(async move {
        for i in 0..10 {
            client
                .publish(STATUS_TOPIC, QoS::AtLeastOnce, true, &[255])
                .await
                .unwrap();
            time::sleep(Duration::from_millis(100)).await;
        }
    });

    let (tx, rx) = mpsc::channel(32);

    let command_task = command_reciever(event_loop);
    let motor_task = motor_driver(rx, close_pin, open_pin);

    tokio::join!(command_task, motor_task);
    println!("Exiting...");
}
