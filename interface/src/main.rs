use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

// Application state
#[derive(Clone)]
struct AppState {
    mqtt_client: Arc<Mutex<AsyncClient>>,
}

// MQTT configuration
struct MqttConfig {
    broker: String,
    port: u16,
    client_id: String,
    blinds_topic: String,
}

// Command for roller blinds
#[derive(Debug, Serialize, Deserialize)]
struct BlindsCommand {
    position: u8,      // 0-100 percentage
    speed: Option<u8>, // Optional speed setting (0-100)
}

// Error type
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("MQTT error: {0}")]
    MqttError(#[from] rumqttc::ClientError),
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::MqttError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::InvalidCommand(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        (status, message).into_response()
    }
}

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // MQTT configuration
    let mqtt_config = MqttConfig {
        broker: "localhost".to_string(),
        port: 1883,
        client_id: "roller_blinds_controller".to_string(),
        blinds_topic: "home/living-room/blinds/control".to_string(),
    };

    // Set up MQTT client
    let mut mqtt_options =
        MqttOptions::new(mqtt_config.client_id, mqtt_config.broker, mqtt_config.port);
    mqtt_options.set_keep_alive(std::time::Duration::from_secs(5));

    let (mqtt_client, mut event_loop) = AsyncClient::new(mqtt_options, 10);

    // Spawn a task to handle MQTT events
    tokio::spawn(async move {
        while let Ok(event) = event_loop.poll().await {
            log::debug!("MQTT Event: {:?}", event);
        }
    });

    // Create application state
    let state = AppState {
        mqtt_client: Arc::new(Mutex::new(mqtt_client)),
    };

    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/api/blinds/control", post(control_blinds))
        .route("/api/blinds/status", get(get_blinds_status))
        .layer(cors)
        .with_state(state);

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    log::info!("Server listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

// Basic root handler
async fn root() -> Html<&'static str> {
    Html("<h1>Roller Blinds Control API</h1><p>Use /api/blinds/control to send commands</p>")
}

// Handler to control the blinds
async fn control_blinds(
    State(state): State<AppState>,
    Json(command): Json<BlindsCommand>,
) -> Result<Json<BlindsCommand>, AppError> {
    // Validate the command
    if command.position > 100 {
        return Err(AppError::InvalidCommand(
            "Position must be between 0 and 100".to_string(),
        ));
    }

    if let Some(speed) = command.speed {
        if speed > 100 {
            return Err(AppError::InvalidCommand(
                "Speed must be between 0 and 100".to_string(),
            ));
        }
    }

    // Serialize the command to JSON
    let payload = serde_json::to_string(&command)?;

    // Publish the command to MQTT
    let client = state.mqtt_client.lock().await;
    client
        .publish(
            "home/living-room/blinds/control", // In a real app, this would come from config
            QoS::AtLeastOnce,
            false,
            payload,
        )
        .await?;

    Ok(Json(command))
}

// Handler to get blinds status (mock implementation)
async fn get_blinds_status() -> Json<BlindsCommand> {
    // In a real application, this would query the current status from MQTT or a database
    Json(BlindsCommand {
        position: 50,    // Mock position
        speed: Some(75), // Mock speed
    })
}
