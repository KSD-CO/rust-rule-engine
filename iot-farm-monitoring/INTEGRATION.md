# Integration Guide

## Integrating IoT Farm Monitoring with Your System

This guide explains how to integrate the IoT Farm Monitoring system with various platforms and services.

## 🔌 Kafka Integration

### Consumer Configuration

```rust
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};

let consumer: StreamConsumer = ClientConfig::new()
    .set("bootstrap.servers", "kafka1:9092,kafka2:9092,kafka3:9092")
    .set("group.id", "farm-monitor-group")
    .set("enable.auto.commit", "true")
    .set("auto.offset.reset", "earliest")
    .set("session.timeout.ms", "6000")
    .set("enable.partition.eof", "false")
    .create()?;
```

### Kafka Schema Registry

For production, use Avro schemas:

```rust
use schema_registry_converter::async_impl::avro::AvroDecoder;

let decoder = AvroDecoder::new(SchemaRegistryConfig {
    urls: vec!["http://schema-registry:8081".to_string()],
});

let decoded = decoder.decode(message_payload).await?;
```

### Kafka Security (SASL/SSL)

```rust
ClientConfig::new()
    .set("security.protocol", "SASL_SSL")
    .set("sasl.mechanism", "PLAIN")
    .set("sasl.username", "your-username")
    .set("sasl.password", "your-password")
    .set("ssl.ca.location", "/path/to/ca-cert")
    .create()?
```

## 🗄️ Database Integration

### PostgreSQL

Add to `Cargo.toml`:
```toml
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres"] }
```

Store events:
```rust
use sqlx::PgPool;

pub async fn store_event(pool: &PgPool, event: &StreamEvent) -> Result<()> {
    sqlx::query!(
        "INSERT INTO sensor_events (id, event_type, zone_id, data, timestamp)
         VALUES ($1, $2, $3, $4, $5)",
        event.id,
        event.event_type,
        event.data.get("zone_id").and_then(|v| v.as_string()),
        serde_json::to_value(&event.data)?,
        event.metadata.timestamp as i64
    )
    .execute(pool)
    .await?;

    Ok(())
}
```

### TimescaleDB (Time-Series)

Create hypertable:
```sql
CREATE TABLE sensor_readings (
    time TIMESTAMPTZ NOT NULL,
    zone_id TEXT NOT NULL,
    sensor_type TEXT NOT NULL,
    value DOUBLE PRECISION,
    metadata JSONB
);

SELECT create_hypertable('sensor_readings', 'time');
```

## 📧 Alerting Integration

### Email Alerts (SMTP)

Add to `Cargo.toml`:
```toml
lettre = "0.11"
```

Send alerts:
```rust
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;

pub async fn send_alert(config: &AlertingConfig, subject: &str, body: &str) -> Result<()> {
    let email = Message::builder()
        .from(config.from_email.parse()?)
        .to(config.to_emails[0].parse()?)
        .subject(subject)
        .body(body.to_string())?;

    let creds = Credentials::new(
        config.smtp_username.clone(),
        config.smtp_password.clone()
    );

    let mailer = SmtpTransport::relay(&config.smtp_server)?
        .credentials(creds)
        .build();

    mailer.send(&email)?;
    Ok(())
}
```

### Webhook Notifications

```rust
use reqwest;

pub async fn send_webhook(webhook_url: &str, payload: serde_json::Value) -> Result<()> {
    let client = reqwest::Client::new();
    client
        .post(webhook_url)
        .json(&payload)
        .send()
        .await?;
    Ok(())
}
```

### Slack Integration

```rust
pub async fn send_slack_alert(webhook_url: &str, message: &str) -> Result<()> {
    let payload = serde_json::json!({
        "text": message,
        "username": "Farm Monitor",
        "icon_emoji": ":tractor:"
    });

    send_webhook(webhook_url, payload).await
}
```

## 📊 Metrics Integration

### Prometheus

Add to `Cargo.toml`:
```toml
prometheus = "0.13"
```

Export metrics:
```rust
use prometheus::{Counter, Histogram, Registry};

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();

    static ref EVENTS_PROCESSED: Counter = Counter::new(
        "farm_events_processed_total",
        "Total number of events processed"
    ).unwrap();

    static ref EVENT_LATENCY: Histogram = Histogram::new(
        "farm_event_processing_seconds",
        "Event processing latency"
    ).unwrap();
}

pub fn init_metrics() -> Result<()> {
    REGISTRY.register(Box::new(EVENTS_PROCESSED.clone()))?;
    REGISTRY.register(Box::new(EVENT_LATENCY.clone()))?;
    Ok(())
}

// In your handler
pub fn process_event_with_metrics(event: StreamEvent) {
    let timer = EVENT_LATENCY.start_timer();

    // Process event...

    EVENTS_PROCESSED.inc();
    timer.observe_duration();
}
```

Expose metrics endpoint:
```rust
use warp::Filter;

#[tokio::main]
async fn main() {
    let metrics_route = warp::path!("metrics")
        .map(|| {
            use prometheus::Encoder;
            let encoder = prometheus::TextEncoder::new();
            let mut buffer = Vec::new();
            encoder.encode(&REGISTRY.gather(), &mut buffer).unwrap();
            String::from_utf8(buffer).unwrap()
        });

    warp::serve(metrics_route).run(([0, 0, 0, 0], 9090)).await;
}
```

## 🎨 Dashboard Integration

### Grafana

Create dashboard JSON:
```json
{
  "dashboard": {
    "title": "IoT Farm Monitoring",
    "panels": [
      {
        "title": "Events Processed",
        "targets": [
          {
            "expr": "rate(farm_events_processed_total[5m])"
          }
        ]
      },
      {
        "title": "Irrigation Triggers",
        "targets": [
          {
            "expr": "farm_irrigation_triggered_total"
          }
        ]
      }
    ]
  }
}
```

### Custom Web UI

Create REST API:
```rust
use warp::Filter;

#[derive(Serialize)]
struct StatsResponse {
    events_processed: u64,
    irrigation_triggered: u64,
    frost_alerts: u64,
}

pub fn api_routes(monitor: Arc<Mutex<FarmMonitor>>)
    -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    let stats_route = warp::path!("api" / "stats")
        .and(warp::get())
        .map(move || {
            let monitor = monitor.lock().unwrap();
            let stats = monitor.get_stats();
            warp::reply::json(&StatsResponse {
                events_processed: stats.events_processed,
                irrigation_triggered: stats.irrigation_triggered,
                frost_alerts: stats.frost_alerts,
            })
        });

    stats_route
}
```

## 🌐 Cloud Integrations

### AWS IoT Core

```rust
use aws_sdk_iot as iot;
use aws_sdk_iotdataplane as iot_data;

pub async fn publish_to_aws_iot(topic: &str, payload: &[u8]) -> Result<()> {
    let config = aws_config::load_from_env().await;
    let client = iot_data::Client::new(&config);

    client
        .publish()
        .topic(topic)
        .qos(1)
        .payload(Blob::new(payload))
        .send()
        .await?;

    Ok(())
}
```

### Azure IoT Hub

```rust
use azure_iot_sdk::client::IotHubClient;

pub async fn send_to_azure_iot(
    connection_string: &str,
    message: &str
) -> Result<()> {
    let client = IotHubClient::from_connection_string(connection_string)?;
    client.send_message(message).await?;
    Ok(())
}
```

### Google Cloud IoT

```rust
use google_cloud_iot::v1::DeviceManagerClient;

pub async fn publish_to_gcp_iot(
    project_id: &str,
    location: &str,
    registry_id: &str,
    device_id: &str,
    data: &[u8]
) -> Result<()> {
    let mut client = DeviceManagerClient::connect("cloudiot.googleapis.com:443").await?;

    // Publish telemetry...

    Ok(())
}
```

## 🔒 Security

### JWT Authentication

```rust
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
```

### RBAC (Role-Based Access Control)

```rust
pub enum Permission {
    ViewSensors,
    TriggerIrrigation,
    ViewAlerts,
    ManageSystem,
}

pub struct User {
    pub id: String,
    pub roles: Vec<String>,
}

pub fn has_permission(user: &User, permission: Permission) -> bool {
    match permission {
        Permission::ViewSensors => true, // Everyone can view
        Permission::TriggerIrrigation => user.roles.contains(&"operator".to_string()),
        Permission::ManageSystem => user.roles.contains(&"admin".to_string()),
        _ => false,
    }
}
```

## 🧪 Testing Integrations

### Mock Kafka Producer

```rust
#[cfg(test)]
pub struct MockKafkaProducer {
    events: Arc<Mutex<Vec<StreamEvent>>>,
}

impl MockKafkaProducer {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn send(&self, event: StreamEvent) -> Result<()> {
        self.events.lock().unwrap().push(event);
        Ok(())
    }

    pub fn get_sent_events(&self) -> Vec<StreamEvent> {
        self.events.lock().unwrap().clone()
    }
}
```

### Integration Test Example

```rust
#[tokio::test]
async fn test_end_to_end_workflow() {
    let config = Config::default();
    let mut monitor = FarmMonitor::new(config);

    monitor.register_irrigation_control();

    // Simulate events
    let soil = create_soil_sensor_reading("zone_1", 20.0, 1000);
    let temp = create_temperature_reading("zone_1", 30.0, "soil", 1010);

    monitor.process_event(soil);
    monitor.process_event(temp);

    let stats = monitor.get_stats();
    assert_eq!(stats.irrigation_triggered, 1);
}
```

## 📖 Additional Resources

- [Kafka Connect](https://kafka.apache.org/documentation/#connect)
- [AWS IoT SDK](https://docs.aws.amazon.com/iot/latest/developerguide/)
- [Prometheus Rust Client](https://docs.rs/prometheus/latest/prometheus/)
- [Grafana Dashboards](https://grafana.com/docs/grafana/latest/dashboards/)
