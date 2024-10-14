BINARY_NAME=hanasu
SERVICE_NAME=$(BINARY_NAME).service
INSTALL_DIR=/usr/local/bin
SYSTEMD_DIR=/etc/systemd/system
USER=$(shell whoami)
WORKING_DIR=$(shell pwd)

all: build install_service

build:
	cargo build --release
	@echo "Build complete."

install_binary: build
	sudo cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Binary installed to $(INSTALL_DIR)/$(BINARY_NAME)."

install_service: install_binary
	@echo "Creating systemd service file..."
	echo "[Unit]" | sudo tee $(SYSTEMD_DIR)/$(SERVICE_NAME)
	echo "Description=Hanasu Audio Service" | sudo tee -a $(SYSTEMD_DIR)/$(SERVICE_NAME)
	echo "After=network.target" | sudo tee -a $(SYSTEMD_DIR)/$(SERVICE_NAME)
	echo "" | sudo tee -a $(SYSTEMD_DIR)/$(SERVICE_NAME)
	echo "[Service]" | sudo tee -a $(SYSTEMD_DIR)/$(SERVICE_NAME)
	echo "ExecStart=$(INSTALL_DIR)/$(BINARY_NAME)" | sudo tee -a $(SYSTEMD_DIR)/$(SERVICE_NAME)
	echo "Restart=always" | sudo tee -a $(SYSTEMD_DIR)/$(SERVICE_NAME)
	echo "RestartSec=5" | sudo tee -a $(SYSTEMD_DIR)/$(SERVICE_NAME)
	echo "User=$(USER)" | sudo tee -a $(SYSTEMD_DIR)/$(SERVICE_NAME)
	echo "WorkingDirectory=$(WORKING_DIR)" | sudo tee -a $(SYSTEMD_DIR)/$(SERVICE_NAME)
	echo "" | sudo tee -a $(SYSTEMD_DIR)/$(SERVICE_NAME)
	echo "[Install]" | sudo tee -a $(SYSTEMD_DIR)/$(SERVICE_NAME)
	echo "WantedBy=multi-user.target" | sudo tee -a $(SYSTEMD_DIR)/$(SERVICE_NAME)

	@echo "Systemd service file created at $(SYSTEMD_DIR)/$(SERVICE_NAME)."

enable_service:
	sudo systemctl enable $(SERVICE_NAME)
	sudo systemctl start $(SERVICE_NAME)
	@echo "Service enabled and started."

uninstall:
	sudo systemctl stop $(SERVICE_NAME)
	sudo systemctl disable $(SERVICE_NAME)
	sudo rm -f $(SYSTEMD_DIR)/$(SERVICE_NAME)
	sudo rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Service uninstalled."

clean:
	cargo clean
	@echo "Clean complete."

.PHONY: all build install_binary install_service enable_service uninstall clean

