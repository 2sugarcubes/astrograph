wasm: 
	# Massively reduce bundle size
	wasm-pack build --target web --release . 

test:
	cargo test

pre-push:
	cargo fmt && \
		cargo clippy --all-features -- -Dclippy::pedantic && \
		cargo clippy --no-default-features && \
		echo "\tf64 tests" && cargo test && \
		echo "\tf32 tests" && cargo test --no-default-features && \
		echo '✅ Good to push 👍'

