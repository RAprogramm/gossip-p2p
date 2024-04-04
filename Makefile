.PHONY: build run

TARGET=./gossip_p2p
BUILD_PATH=target/debug/gossip_p2p
LOCALHOST=127.0.0.1:

$(TARGET): $(BUILD_PATH)
	cp $(BUILD_PATH) $(TARGET)

$(BUILD_PATH):
	cargo build

build:
	cargo build --release
	cp $(BUILD_PATH) $(TARGET)

run: $(TARGET)
	$(TARGET) --period=$(TICK) --port=$(FROM) $(if $(TO),--connect=$(LOCALHOST)$(TO))
