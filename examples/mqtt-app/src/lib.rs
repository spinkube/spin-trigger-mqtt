use chrono::{DateTime, Utc};

wit_bindgen::generate!({
    world: "spin-mqtt",
    path: "../../",
    exports: {
        world: SpinMqtt
    }
});

struct SpinMqtt;

impl Guest for SpinMqtt {    
    fn handle_message(message: Vec<u8>) {               
        let datetime: DateTime<Utc> = std::time::SystemTime::now().into();
        let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string();

        println!(
            "{:?} Message received by wasm component: '{}'",
            formatted_time,
            String::from_utf8_lossy(&message)
        );
    }
}
