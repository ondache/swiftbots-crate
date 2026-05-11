check:
	cargo check --features default
	cargo check --features default --target wasm32-unknown-unknown
	cargo check --features basic
	cargo check --features basic --target wasm32-unknown-unknown
	cargo check --features chat
	cargo check --features chat --target wasm32-unknown-unknown
	cargo check --features telegram
	cargo check --features telegram --target wasm32-unknown-unknown
	cargo check --all-features
	cargo check --all-features --target wasm32-unknown-unknown
	cargo build --all-features
	cargo build --all-features --target wasm32-unknown-unknown
	cargo clippy --features default
	cargo clippy --features basic
	cargo clippy --features chat
	cargo clippy --features telegram
	cargo clippy --features full
	cargo test