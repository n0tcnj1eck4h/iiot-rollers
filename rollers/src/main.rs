mod encoder;
mod motor;

use std::{error::Error, thread, time::Duration};

use encoder::{EncoderPin, handle_encoder_interrupt};
use rppal::gpio::{Gpio, Trigger};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use tokio::{io::AsyncBufReadExt, sync::mpsc::channel, time::sleep};

use crate::{
    encoder::{get_encoder_count, reset_encoder_count},
    motor::Motor,
};

const COMMAND_TOPIC: &str = "shutters/desired";
const STATUS_TOPIC: &str = "shutters/status";

const CLIENT_ID: &str = "apparatus";
const HOST: &str = "localhost";
const PORT: u16 = 1883;

enum MotorCommand {
    Go(i32),
    Calibrate,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new().unwrap();

    // Motor Controller
    let mut motor = Motor {
        a: gpio.get(12)?.into_output_low(),
        b: gpio.get(13)?.into_output_low(),
    };
    motor.stop();

    let (tx, mut rx) = channel(128);
    tokio::spawn(async move {
        while let Some(command) = rx.recv().await {
            match command {
                MotorCommand::Go(desired_pos) => {
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
                MotorCommand::Calibrate => {
                    reset_encoder_count();
                    motor.open();
                    thread::sleep(Duration::from_millis(100));
                    motor.stop();
                    if get_encoder_count() < 0 {
                        motor.flip_polarity();
                    }
                }
            }
        }
    });

    let mut mqttoptions = MqttOptions::new(CLIENT_ID, HOST, PORT);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let (client, mut event_loop) = AsyncClient::new(mqttoptions, 10);
    client
        .subscribe(COMMAND_TOPIC, QoS::AtMostOnce)
        .await
        .unwrap();

    // Receieve
    {
        let tx = tx.clone();
        tokio::task::spawn(async move {
            while let Ok(notification) = event_loop.poll().await {
                if let Event::Incoming(packet) = notification {
                    if let Packet::Publish(publish) = packet {
                        dbg!(publish.payload.len());
                        // let bytes = publish.payload.get(0..3).unwrap();
                        // let desired_value = i32::from_le_bytes(bytes.try_into().unwrap());
                        let v: i32 = std::str::from_utf8(&publish.payload)
                            .unwrap()
                            .parse()
                            .unwrap();
                        tx.send(MotorCommand::Go(v)).await.unwrap();
                    }
                }
            }
        });
    }

    // Encoder
    let mut clk = gpio.get(2)?.into_input();
    let mut dt = gpio.get(3)?.into_input();
    let debounce = None;
    clk.set_async_interrupt(Trigger::Both, debounce, |event| {
        handle_encoder_interrupt(EncoderPin::CLK, event);
    })?;
    dt.set_async_interrupt(Trigger::Both, debounce, |event| {
        handle_encoder_interrupt(EncoderPin::DT, event);
    })?;

    // Publish
    let mut last_encoder_state = 0;
    tokio::spawn(async move {
        loop {
            let encoder_state = get_encoder_count();
            if encoder_state != last_encoder_state {
                client
                    .publish(
                        STATUS_TOPIC,
                        QoS::AtLeastOnce,
                        true,
                        encoder_state.to_string().into_bytes(),
                    )
                    .await
                    .unwrap();
                last_encoder_state = encoder_state;
            }
            sleep(Duration::from_millis(100)).await;
        }
    });

    // CLI
    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin).lines();
    while let Some(line) = reader.next_line().await? {
        let mut split = line.as_str().trim().split(char::is_whitespace);
        let command = split.next().unwrap();
        match command {
            "calibrate" => {
                tx.send(MotorCommand::Calibrate).await.unwrap();
            }
            "enc" => println!("Encoder: {}", get_encoder_count()),
            "go" => {
                let arg: Option<i32> = split.next().map(str::parse).map(Result::ok).flatten();
                if let Some(desired_pos) = arg {
                    tx.send(MotorCommand::Go(desired_pos)).await.unwrap();
                }
            }
            "exit" => break,
            _ => println!("Unknown command"),
        }
    }
    println!("Exiting...");
    Ok(())
}
