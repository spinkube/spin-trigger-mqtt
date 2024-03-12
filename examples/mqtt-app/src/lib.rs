use spin_mqtt_sdk::{mqtt_component, Payload};
use chrono::{DateTime, Utc};

#[mqtt_component]
fn handle_message(message: Payload) -> anyhow::Result<()> {
    let datetime: DateTime<Utc> = std::time::SystemTime::now().into();
    let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string();

    println!(
        "{:?} Message received by wasm component: '{}'",
        formatted_time,
        String::from_utf8_lossy(&message)
    );
    Ok(())
}