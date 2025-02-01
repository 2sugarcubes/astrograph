#!/bin/zsh
function fail {
  echo "$@ do not push yet ❌"
  exit 1
}

echo -e " ✒️ running rust format" &&
  cargo fmt &&
  echo -e "\n 📦 Building the solar-system.program.json" &&
  cd assets &&
  ./packSolarSystem.sh &&
  cd - ||
  fail "Could not pack solar system 🫗" &&
  if command -v typos > /dev/null;
  then
    typos --locale en-us && echo "Spelling appears to be all correct" || 
    fail 'run `typos -w --locale en-us` to auto-accept all changes'; 
  else ;
    echo "You could find spelling errors if you installed [typos](https://github.com/crate-ci/typos#install)"; 
  fi &&
  echo -e "\n 🔎 checking pedantically on f64" &&
  cargo clippy --all-targets --all-features -- -Dclippy::pedantic -Dwarnings &&
  echo -e "\n 🔎 checking pedantically on f32" &&
  cargo clippy --all-targets --no-default-features -- -Dclippy::pedantic -Dwarnings ||
  fail "Clippy did not like that 🤬" &&
  echo -e "\n 🧪 Running tests for f32 and f64" &&
  echo -e "\tf64 tests" && cargo test --all-features &&
  echo &&
  echo -e "\tf32 tests" && cargo test --frozen --no-default-features ||
  fail "Test(s) did not pass 🧪" &&
  echo -e "\n 🕸️ Building the wasm target" &&
  wasm-pack build --target web --no-opt -d ../web/pkg wasm ||
  fail "Could not build wasm 🛜" &&
  echo -e "\n 🌟 Test run to generate a universe" &&
  cargo run -- -vv build -c 100 -s 0x100000000000000000000 &&
  rm universe.json &&
  echo -e "\n 💫 Test run with full Program" &&
  cargo run -- -vv -o /tmp/astrolabe simulate -s 100 -e 200 -t 10 -p assets/solar-system.program.json ||
  fail "Program from full json failed " &&
  echo -e "\n 💫 Test run with partial program" &&
  cargo run -- -vv -o /tmp/astrolabe simulate -s 0 -e 100 -t 10 -u assets/solar-system.json -o assets/solar-system.observatories.json ||
  fail "Program from parts failed " &&
  echo -e "\n ☂️ Running code coverage" &&
  cargo tarpaulin --skip-clean --fail-under 50 --exclude-files '**/main.rs' --frozen --out html | tail -n 1 &&
  echo '✅ Good to push 👍'
