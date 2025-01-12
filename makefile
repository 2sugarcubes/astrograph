wasm: 
	# Massively reduce bundle size
	wasm-pack build --target web --release . -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort

test:
	cargo test

pre-push:
	cargo fmt && cargo clippy -- -Dclippy::pedantic && cargo test && echo '✅ Good to push 👍'

