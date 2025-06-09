mod db;

use std::{sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
};
use bincode::{Decode, Encode};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::db::Database;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState::new().await?;
    let app = Router::new()
        .route("/", get(root))
        .route("/style.css", get(css))
        .route("/script.js", get(script))
        .route("/shutters", get(get_shutters))
        .route("/shutters", put(set_shutter))
        .route("/shutters", post(add_shutter))
        .route("/shutters", delete(delete_shutter))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Listening on: http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct ShutterState {
    id: u64,
    position: i32,
}

#[derive(Deserialize)]
struct AddShutterRequest {
    name: String,
    command_topic: String,
    status_topic: String,
}

#[derive(Deserialize)]
struct DeleteShutterRequest {
    id: u64,
}

async fn root() -> impl IntoResponse {
    Html(include_str!("../static/index.html"))
}

async fn css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        include_str!("../static/style.css"),
    )
}

async fn script() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        include_str!("../static/script.js"),
    )
}

async fn add_shutter(
    State(app): State<AppState>,
    Json(payload): Json<AddShutterRequest>,
) -> impl IntoResponse {
    let id = app.db.generate_id();
    let shutter = Shutter {
        id,
        name: payload.name,
        command_topic: payload.command_topic,
        status_topic: payload.status_topic,
        position: 100,
    };

    let client = app.client.lock().await;
    client
        .subscribe(&shutter.status_topic, QoS::AtLeastOnce)
        .await
        .unwrap();

    app.db.insert_shutter(shutter).unwrap();
    StatusCode::OK
}

async fn get_shutters(State(app): State<AppState>) -> impl IntoResponse {
    let response: Vec<_> = app.db.iter_shutters().collect();
    Json(response)
}

async fn set_shutter(
    State(app): State<AppState>,
    Json(payload): Json<ShutterState>,
) -> impl IntoResponse {
    let shutter = app.db.get_shutter(payload.id).unwrap().unwrap();
    let client = app.client.lock().await;
    client
        .publish(
            &shutter.command_topic,
            QoS::AtLeastOnce,
            false,
            payload.position.to_string().as_bytes(),
        )
        .await
        .unwrap();
    StatusCode::OK
}

async fn delete_shutter(
    State(app): State<AppState>,
    Json(payload): Json<DeleteShutterRequest>,
) -> impl IntoResponse {
    let shutter = app.db.get_shutter(payload.id).unwrap().unwrap();
    let client = app.client.lock().await;
    client.unsubscribe(shutter.status_topic).await.unwrap();
    app.db.remove_shutter(payload.id).unwrap();
    StatusCode::OK
}

const CLIENT_ID: &str = "interface";
const HOST: &str = "192.168.0.10";
const PORT: u16 = 1883;

#[derive(Encode, Decode, Debug, Serialize)]
struct Shutter {
    id: u64,
    name: String,
    command_topic: String,
    status_topic: String,
    position: i32,
}

#[derive(Clone)]
struct AppState {
    client: Arc<Mutex<AsyncClient>>,
    db: Arc<db::Database>,
}

impl AppState {
    async fn new() -> anyhow::Result<Self> {
        let mut mqttoptions = MqttOptions::new(CLIENT_ID, HOST, PORT);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let (client, mut event_loop) = AsyncClient::new(mqttoptions, 64);

        let kv = Arc::new(db::Database::new());
        {
            let kv = kv.clone();
            tokio::task::spawn(async move {
                println!("MQTT event loop starting...");
                while let Ok(notification) = event_loop.poll().await {
                    if let Err(err) = handle_mqtt_notification(&kv, notification) {
                        eprintln!("Error handling MQTT notification: {}", err);
                    }
                }
                println!("MQTT event loop shutting down...");
            });
        }

        for shutter in kv.iter_shutters() {
            client
                .subscribe(shutter.status_topic, QoS::AtLeastOnce)
                .await?;
        }

        Ok(Self {
            db: kv,
            client: Arc::new(Mutex::new(client)),
        })
    }
}

fn handle_mqtt_notification(kv: &Database, notification: Event) -> anyhow::Result<()> {
    if let Event::Incoming(packet) = notification {
        if let Packet::Publish(publish) = packet {
            let topic = publish.topic;
            let v: i32 = std::str::from_utf8(&publish.payload)
                .unwrap()
                .parse()
                .unwrap();

            let mut shutter = kv.get_shutter_by_status_topic(&topic).unwrap();
            shutter.position = v;
            kv.insert_shutter(shutter)?;
        }
    }
    Ok(())
}
