// use fermyon::spin::variables;

use exports::spin::mqtt_trigger_sdk::inbound_mqtt::Guest;
// use spin_core::async_trait;
use spin::mqtt_trigger_sdk::{
    mqtt_types::Error,
    outbound_mqtt,    
};

wit_bindgen::generate!({
    world: "spin-mqtt",
    path: "../../spin-mqtt.wit",
    exports: {
        "spin:mqtt-trigger-sdk/inbound-mqtt": SpinMqtt
    }
});

struct SpinMqtt;

// #[async_trait]
impl Guest for SpinMqtt {
    fn handle_message(message: Vec<u8>) -> Result<(), Error> {
        println!("Message received by wasm component: {}", String::from_utf8_lossy(&message));
        // Add Spin config wit to extract config values from spin.toml e.g. topic
        
        // Echo message back to the Mqtt broker.        
        // outbound_mqtt::publish("message-out", &message)?;

        Ok(())
    }
}
