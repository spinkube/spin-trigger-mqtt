use clap::Parser;
use spin_trigger::cli::TriggerExecutorCommand;
use trigger_mqtt::MqttTrigger;
use std::io::IsTerminal;

type Command = TriggerExecutorCommand<MqttTrigger>;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(std::io::stderr().is_terminal())
        .init();

    let trigger = Command::parse();
    trigger.run().await
}
