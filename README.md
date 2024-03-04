# An MQTT Trigger/Plugin for Fermyon Spin Framework

MQTT is a dominant communication protocol in IoT and edge scenarios, used by major products and services in manufacturing, automotive and other industries.
Objective of this repo is to provide a robust plugin/trigger to receive MQTT messages in the Spin based wasm components.

## Usage Guidance

This plugin is a trigger plugin i.e. it is activated when message is received on a configured MQTT topic.
The plugin then instantiates a Wasm component and injects the message to the component, which in turn process the message and can optionally publish the messages to any of the available targets in Spin e.g. MQTT, Redis, Http endpoints.

### Install Plugin

Install MQTT Plugin:

`spin plugin install --url https://raw.githubusercontent.com/suneetnangia/spin-mqtt-trigger-sdk/main/trigger-mqtt.json --yes`

[Note: release management for multiple versions of this plugin/trigger will be added soon]

If you want to learn more about Spin's plugin model, read [here](https://www.fermyon.com/blog/managing-spin-templates-and-plugins).

### Install Template

[Spin templates](https://www.fermyon.com/blog/managing-spin-templates-and-plugins) allow a Spin developer to quickly create the skeleton of an application or component, ready for the application logic to be filled in. As part of this repo, a new template is created to help build applications which make use of MQTT as a communication protocol/trigger.

Install MQTT Template:

`spin templates install --git https://github.com/suneetnangia/spin-mqtt-trigger-sdk --upgrade`

### Create Spin App

`spin new mqtt-app`

Select the template called `mqtt-rust`

## State of Play

1. Authenticates using anonymous and username/password to MQTT server.
2. Receive messages from an MQTT topic per configured QoS.
3. Template way of installing trigger/plugin needs fixing.
[more MQTT client/subscription attributes will be available soon]

## Dev Loop

* Install `Spin` from [here](https://developer.fermyon.com/spin/v2/install).
* Run `make` to build and install the plugin locally.
* Run `spin build --up --from examples/mqtt-app/spin.toml` to run example Spin app.
