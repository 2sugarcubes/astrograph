.PHONY: help

wasm: ## Build wasm target
	# Massively reduce bundle size
	@wasm-pack build --target web --release -d pkg wasm

test: ## Run cargo tests
	@cargo test

pre-push: ## Run all the CI tests (With the exception of tests that run on a different OS, i.e. windows and/or ubuntu).
	@cargo fmt && \
		cd assets && \
		./packSolarSystem.sh && \
		cd - && \
		cargo clippy --all-targets --all-features -- -Dclippy::pedantic -Dwarnings && \
		cargo clippy --all-targets --no-default-features -- -Dclippy::pedantic -Dwarnings && \
		echo "\tf64 tests" && cargo test --all-features && \
		echo "\tf32 tests" && cargo test --no-default-features && \
		wasm-pack build --target web -d pkg wasm && \
		cargo run -- build -c 100 -s 0x100000000000000000000 && \
		rm universe.json && \
		cargo run -- -o /tmp/astrolabe simulate -s 100 -e 200 -t 10 -p assets/solar-system.program.json && \
		cargo run -- -o /tmp/astrolabe simulate -s 0 -e 100 -t 10 -u assets/solar-system.json -o assets/solar-system.observatories.json && \
		echo '✅ Good to push 👍'

serve: ## Build and serve wasm on a testing server
	@make wasm && cd ./web/ && python -m http.server; cd ..

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
