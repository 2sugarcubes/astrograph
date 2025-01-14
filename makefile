wasm: 
	# Massively reduce bundle size
	wasm-pack build --target web --release -d ../web/pkg lib

test:
	cargo test

pre-push:
	cargo fmt && \
		cd assets && \
		./packSolarSystem.sh && \
		cd - && \
		cargo clippy --all-features -- -Dclippy::pedantic && \
		cargo clippy --no-default-features && \
		echo "\tf64 tests" && cargo test && \
		echo "\tf32 tests" && cargo test --no-default-features && \
		cargo run -- build -c 100 -s 0x100000000000000000000 && \
		rm universe.json && \
		cargo run -- -o /tmp/astrolabe simulate -s 0 -e 100 -t 10 -u assets/solar-system.json -o assets/solar-system.observatories.json && \
		cargo run -- -o /tmp/astrolabe simulate -s 100 -e 200 -t 10 -p assets/solar-system.program.json && \
		echo '✅ Good to push 👍'

serve:
	make wasm && cd ./web/ && python -m http.server; cd ..
