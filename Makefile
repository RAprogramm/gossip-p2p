build:
	cargo build && cp target/debug/peer ./

start_peer_1:
	cargo run -- --period=5 --port=8080

start_peer_2:
	cargo run -- --period=6 --port=8081 --connect="127.0.0.1:8080"

start_peer_3:
	cargo run -- --period=7 --port=8082 --connect="127.0.0.1:8080"
