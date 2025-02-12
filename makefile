.PHONY: help wasm test pre-push serve portable-build publish

wasm: ## Build wasm target
	@wasm-pack build --target web --release -d ../web/pkg --out-name astrograph wasm

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

publish-check:
	@make pre-push &&\
		if command -v cargo-semver-checks > /dev/null; \
		then cargo semver-checks --exclude astrograph-wasm && cargo semver-checks -p astrograph-wasm --baseline-rev v0.1.0 || exit 1 \
		else; echo "You can check your semantic versioning here by installing cargo-semver-checks (https://github.com/obi1kenobi/cargo-semver-checks#quick-start)";\
		fi &&\
		cargo publish -np astrograph &&\
		cargo publish -np astrograph-bin
