use anyhow::{anyhow, Context};
use clap::Args;
use paho_mqtt::AsyncClient;
use serde::{Deserialize, Serialize};
use spin_app::MetadataKey;
use spin_core::{async_trait, InstancePre};
use spin_trigger::{TriggerAppEngine, TriggerExecutor};
use std::sync::Arc;
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
#[derive(Clone)]
pub struct MqttTrigger {
    engine: Arc<TriggerAppEngine<Self>>,
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
    type InstancePre = InstancePre<RuntimeData>;

    async fn new(engine: spin_trigger::TriggerAppEngine<Self>) -> anyhow::Result<Self> {
        let address = resolve_template_variable(
            &engine,
            engine.app().require_metadata(TRIGGER_METADATA_KEY)?.address,
        )?;
        let username = resolve_template_variable(
            &engine,
            engine
                .app()
                .require_metadata(TRIGGER_METADATA_KEY)?
                .username,
        )?;
        let password = resolve_template_variable(
            &engine,
            engine
                .app()
                .require_metadata(TRIGGER_METADATA_KEY)?
                .password,
        )?;
        let keep_alive_interval = engine
            .app()
            .require_metadata(TRIGGER_METADATA_KEY)?
            .keep_alive_interval
            .parse::<u64>()?;

        let component_configs =
            engine
                .trigger_configs()
                .try_fold(vec![], |mut acc, (_, config)| {
                    let component = config.component.clone();
                    let qos = config.qos.parse::<i32>()?;
                    let topic = resolve_template_variable(&engine, config.topic.clone())?;
                    acc.push((component, qos, topic));
                    anyhow::Ok(acc)
                })?;

        Ok(Self {
            engine: Arc::new(engine),
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
                self.handle_mqtt_event(&component.0, b"test message".to_vec())
                    .await?;
            }

            Ok(())
        } else {
            tokio::spawn(async move {
                // This trigger spawns threads, which Ctrl+C does not kill. So
                // for this case we need to detect Ctrl+C and shut those threads
                // down. For simplicity, we do this by terminating the process.
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to listen for Ctrl+C");
                std::process::exit(0);
            });

            let tasks: Vec<_> = self
                .component_configs
                .clone()
                .into_iter()
                .map(|(component_id, mqtt_qos, mqtt_topic)| {
                    let trigger = self.clone();
                    tokio::spawn(async move {
                        trigger
                            .run_listener(component_id.clone(), mqtt_qos, mqtt_topic.clone())
                            .await
                    })
                })
                .collect();

            // wait for the first handle to be returned and drop the rest
            let (result, _, rest) = futures::future::select_all(tasks).await;

            drop(rest);
            result?
        }
    }
}

impl MqttTrigger {
    async fn handle_mqtt_event(&self, component_id: &str, message: Vec<u8>) -> anyhow::Result<()> {
        // Load the guest wasm component
        let (instance, mut store) = self.engine.prepare_instance(component_id).await?;

        // SpinMqtt is auto generated by bindgen as per WIT files referenced above.
        let instance = SpinMqtt::new(&mut store, &instance)?;

        instance
            .call_handle_message(store, &message)
            .await?
            .map_err(|err| anyhow!("failed to execute guest: {err}"))
    }

    async fn run_listener(
        &self,
        component_id: String,
        qos: i32,
        topic: String,
    ) -> anyhow::Result<()> {
        // Receive the messages here from the specific topic in mqtt broker.
        let mut client = AsyncClient::new(self.address.clone())?;
        let conn_opts = paho_mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(self.keep_alive_interval))
            .user_name(&self.username)
            .password(&self.password)
            .finalize();

        client
            .connect(conn_opts)
            .await
            .context(format!("failed to connect to {:?}", self.address))?;
        client
            .subscribe(&topic, qos)
            .await
            .context(format!("failed to subscribe to {topic:?}"))?;

        // Should the buffer be bounded/configurable?
        let rx = client.get_stream(None);

        loop {
            match rx.recv().await {
                Ok(Some(msg)) => {
                    // Handle the received message
                    _ = self
                        .handle_mqtt_event(&component_id, msg.payload().to_vec())
                        .await
                        .map_err(|err| tracing::error!("{err}"));
                }
                Ok(None) => {
                    // Todo: Figure out what this case is
                }
                Err(_) => {
                    // Channel is empty and closed
                    break;
                }
            }
        }

        Ok(())
    }
}

fn resolve_template_variable(
    engine: &TriggerAppEngine<MqttTrigger>,
    template_string: String,
) -> anyhow::Result<String> {
    let template_expr = spin_expressions::Template::new(template_string)?;
    anyhow::Ok(engine.resolve_template(&template_expr)?)
}
