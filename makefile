.PHONY: help wasm test pre-push serve portable-build

wasm: ## Build wasm target
	@wasm-pack build --target web --release -d ../web/pkg --out-name astrolabe wasm

test: ## Run cargo tests
	@cargo test

pre-push: ## Run all the CI tests (With the exception of tests that run on a different OS, i.e. windows and/or ubuntu).
	@./tools/push-check.sh

serve: ## Build and serve wasm on a testing server
	@make wasm && cd ./web/ && python -m http.server; cd ..

portable-build: ## Build targets for a release tag
	@tools/build-targets.sh

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
