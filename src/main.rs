use clap::Parser;
use spin_trigger::cli::TriggerExecutorCommand;
use trigger_mqtt::MqttTrigger;

type Command = TriggerExecutorCommand<MqttTrigger>;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let trigger = Command::parse();
    trigger.run().await
}
