use anyhow::{anyhow, Ok};
use clap::Args;
use paho_mqtt::Client;
use serde::{Deserialize, Serialize};
use spin_app::MetadataKey;
use spin_core::async_trait;
use spin_trigger::{EitherInstance, TriggerAppEngine, TriggerExecutor};
use std::time::Duration;

// https://docs.rs/wasmtime/latest/wasmtime/component/macro.bindgen.html
wasmtime::component::bindgen!({
    path: ".",
    world: "spin-mqtt",
    async: true,
});

pub(crate) type RuntimeData = ();
pub(crate) type _Store = spin_core::Store<RuntimeData>;

#[derive(Args)]
pub struct CliArgs {
    /// If true, run each component once and exit
    #[clap(long)]
    pub test: bool,
}

// The trigger structure with all values processed and ready
pub struct MqttTrigger {
    engine: TriggerAppEngine<Self>,
    address: String,
    username: String,
    password: String,
    keep_alive_interval: u64,
    component_configs: Vec<(String, i32, String)>,
}

// Application settings (raw serialization format)
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriggerMetadata {
    r#type: String,
    address: String,
    username: String,
    password: String,
    keep_alive_interval: String,
}

// Per-component settings (raw serialization format)
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MqttTriggerConfig {
    component: String,
    topic: String,
    qos: String,
}

const TRIGGER_METADATA_KEY: MetadataKey<TriggerMetadata> = MetadataKey::new("trigger");

#[async_trait]
impl TriggerExecutor for MqttTrigger {
    const TRIGGER_TYPE: &'static str = "mqtt";
    type RuntimeData = RuntimeData;
    type TriggerConfig = MqttTriggerConfig;
    type RunConfig = CliArgs;

    async fn new(engine: spin_trigger::TriggerAppEngine<Self>) -> anyhow::Result<Self> {
        let address = engine.app().require_metadata(TRIGGER_METADATA_KEY)?.address;
        let username = engine
            .app()
            .require_metadata(TRIGGER_METADATA_KEY)?
            .username;
        let password = engine
            .app()
            .require_metadata(TRIGGER_METADATA_KEY)?
            .password;
        let keep_alive_interval = engine
            .app()
            .require_metadata(TRIGGER_METADATA_KEY)?
            .keep_alive_interval
            .parse::<u64>()?;

        let component_configs = engine
            .trigger_configs()
            .map(|(_, config)| {
                (
                    config.component.clone(),
                    config.qos.parse::<i32>().unwrap(),
                    config.topic.clone(),
                )
            })
            .collect();

        Ok(Self {
            engine,
            address,
            username,
            password,
            keep_alive_interval,
            component_configs,
        })
    }

    async fn run(self, config: Self::RunConfig) -> anyhow::Result<()> {
        if config.test {
            for component in &self.component_configs {
                self.handle_mqtt_event(&component.0, "test message").await?;
            }
        } else {
            tokio::spawn(async move {
                // This trigger spawns threads, which Ctrl+C does not kill. So
                // for this case we need to detect Ctrl+C and shut those threads
                // down. For simplicity, we do this by terminating the process.
                tokio::signal::ctrl_c().await.unwrap();
                std::process::exit(0);
            });

            tokio_scoped::scope(|scope| {
                // TODO: remove unwrap to avoid panics and configure any retries etc.
                for (component_id, mqtt_qos, mqtt_topic) in &self.component_configs {
                    scope.spawn(async {
                        // Receive the messages here from the specific topic in mqtt broker.
                        let client = Client::new(self.address.clone()).unwrap();
                        let conn_opts = paho_mqtt::ConnectOptionsBuilder::new()
                            .keep_alive_interval(Duration::from_secs(self.keep_alive_interval))
                            .user_name(&self.username)
                            .password(&self.password)
                            .finalize();

                        client.connect(conn_opts).unwrap();
                        client.subscribe(mqtt_topic, *mqtt_qos).unwrap();

                        for msg in client.start_consuming() {
                            if let Some(msg) = msg {
                                _ = self
                                    .handle_mqtt_event(component_id, &msg.payload_str())
                                    .await
                                    .map_err(|err| tracing::error!("{err}"));
                            } else {
                                continue;
                            }
                        }
                    });
                }
            });
        }

        Ok(())
    }
}

impl MqttTrigger {
    async fn handle_mqtt_event(&self, component_id: &str, message: &str) -> anyhow::Result<()> {
        // Load the guest wasm component
        let (instance, mut store) = self.engine.prepare_instance(component_id).await?;

        let EitherInstance::Component(instance) = instance else {
            unreachable!()
        };

        // SpinMqtt is auto generated by bindgen as per WIT files referenced above.
        let instance = SpinMqtt::new(&mut store, &instance)?;

        instance
            .call_handle_message(store, &message.as_bytes().to_vec())
            .await?
            .map_err(|err| anyhow!("failed to execute guest: {err}"))
    }
}
