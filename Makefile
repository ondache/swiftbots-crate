check:
	cargo build --all-features
	cargo check --features default
	cargo check --features basic
	cargo check --features chat
	cargo check --features telegram
	cargo check --features full
	cargo clippy --features default
	cargo clippy --features basic
	cargo clippy --features chat
	cargo clippy --features telegram
	cargo clippy --features full
	cargo check --features default --target wasm32-unknown-unknown
	cargo check --features basic --target wasm32-unknown-unknown
	cargo check --features chat --target wasm32-unknown-unknown
	cargo check --features telegram --target wasm32-unknown-unknown
	cargo check --features full --target wasm32-unknown-unknown
	cargo test