# Makefile to build and run MQTT plugin for Spin framework.

.PHONY: all
all: build_plugin package_plugin install_plugin

.PHONY: build_plugin
build_plugin:
	@echo "Building Mqtt Plugin..."
	cargo fmt --check && cargo build --release

.PHONY: package_plugin
package_plugin:
	@echo "Packaging Mqtt Plugin in Spin..."
	# Copy the binary to the root directory
	cp ./target/release/trigger-mqtt .
	# Create a tarball
	(tar czvf trigger-mqtt.tar.gz trigger-mqtt)
		
.PHONY: install_plugin
install_plugin:
	@echo "Installing Mqtt Plugin in Spin..."
	$(eval SHA256 := $(shell shasum -a 256 trigger-mqtt.tar.gz | cut -d ' ' -f 1))
	@echo "Updating sha256 in plugin manfiest..."
	sed -i 's/"sha256": "[a-f0-9]\{64\}"/"sha256": "$(SHA256)"/' ./trigger-mqtt.json
	spin plugin uninstall trigger-mqtt && spin plugin upgrade --file ./trigger-mqtt.json --yes
	spin plugins list --installed | grep trigger-mqtt

	rm trigger-mqtt.tar.gz
	rm trigger-mqtt

.PHONY: clean
clean:
	@echo "Cleaning up..."
	cargo clean
	cargo clean --manifest-path ./examples/mqtt-app/Cargo.toml
	spin plugin uninstall trigger-mqtt