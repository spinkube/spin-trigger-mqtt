# An MQTT Plugin for Fermyon Spin Framework

Objective of this repo is to provide a robust plugin to both receive and send MQTT messages from Spin based wasm components.
MQTT is a dominant communication protocol in IoT and edge scenarios, used by major products and services in manufacturing, automotive and other industries.

## Usage Guidance

### Install Plugin

A new Spin plugin is developed to receive and send messages from Spin apps.

Install MQTT Plugin:

`spin plugin install --url https://github.com/suneetnangia/spin-mqtt-trigger-sdk/releases/download/trigger-mqtt.json --yes`

If you want to learn more about Spin's plugin model, read [here](https://www.fermyon.com/blog/managing-spin-templates-and-plugins).

### Install Template

[Spin templates](https://www.fermyon.com/blog/managing-spin-templates-and-plugins) allow a Spin developer to quickly create the skeleton of an application or component, ready for the application logic to be filled in. As part of this repo, a new template is created to help build applications which make use of MQTT as a communication protocol.

Install MQTT Template:

`spin templates install --git https://github.com/suneetnangia/spin-mqtt-trigger-sdk --upgrade`

### Create Spin App

`spin new mqtt-app`

Select the template called `mqtt-rust`

## State of Play

1. Send and Receive Messages
2. MQTT QoS Support
3. Min Spin version supported 1.4?

## Dev Loop

* `cargo build --release`
* Copy: `cp ./target/release/spin-mqtt-trigger-sdk .`
* `tar czvf spin-mqtt-trigger-sdk.tar.gz ./target/release/spin-mqtt-trigger-sdk`
* Update the plugin manifest (`spin-mqtt-trigger-sdk.json`):
  * Get the SHA: `shasum -a 256 spin-mqtt-trigger-sdk.tar.gz` and copy it into the `sha256` field
  * Update the URL too, to reflect the directory where the tar file is
* `spin plugin install --file ./spin-mqtt-trigger-sdk.json --yes`

# (cargo build --release) && (cp ./target/release/trigger-orchestrator .) && (tar czvf trigger-orchestrator.tar.gz trigger-orchestrator) && (shasum -a 256 trigger-orchestrator.tar.gz)
# Then you should be able to `spin build --up` the guest.
