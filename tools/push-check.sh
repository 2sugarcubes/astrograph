#!/bin/zsh
echo -e " ✒️ running rust format" &&
  cargo fmt &&
  echo -e "\n 📦 Building the solar-system.program.json" &&
  cd assets &&
  ./packSolarSystem.sh &&
  cd - &&
  echo -e "\n 🔎 checking pedantically on f64" &&
  cargo clippy --all-targets --all-features -- -Dclippy::pedantic -Dwarnings &&
  echo -e "\n 🔎 checking pedantically on f32" &&
  cargo clippy --all-targets --no-default-features -- -Dclippy::pedantic -Dwarnings &&
  echo -e "\n 🧪 Running tests for f32 and f64" &&
  echo -e "\tf64 tests" && cargo test --all-features &&
  echo &&
  echo -e "\tf32 tests" && cargo test --frozen --no-default-features &&
  echo -e "\n 🕸️ Building the wasm target" &&
  wasm-pack build --target web --no-opt -d ../web/pkg wasm &&
  echo -e "\n 🌟 Test run to generate a universe" &&
  cargo run -- -vv build -c 100 -s 0x100000000000000000000 &&
  rm universe.json &&
  echo -e "\n 💫 Test run with full Program" &&
  cargo run -- -vv -o /tmp/astrolabe simulate -s 100 -e 200 -t 10 -p assets/solar-system.program.json &&
  echo -e "\n 💫 Test run with partial program" &&
  cargo run -- -vv -o /tmp/astrolabe simulate -s 0 -e 100 -t 10 -u assets/solar-system.json -o assets/solar-system.observatories.json &&
  echo -e "\n ☂️ Running code coverage" &&
  cargo tarpaulin --skip-clean --fail-under 50 --exclude-files */main.rs --frozen --out html | tail -n 1 &&
  echo '✅ Good to push 👍'
