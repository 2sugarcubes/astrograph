#!/bin/bash

out_dir='.build_artifacts'

#for target in {{x86_64,i686}-{unknown-linux,pc-windows}-gnu,x86_64-apple-darwin,aarch64-{unknown-linux-gnu,apple-darwin}}; do
# build for x86_64 (windows, and linux), i686 (windows, and linux), and ARM32 (linux, raspberry pi)
for target in {{x86_64,i686}-{unknown-linux,pc-windows}-gnu,arm-unknown-linux-gnueabihf}; do
  # build for x86_64 (windows, and linux), i686 (windows, and linux)
  #for target in {x86_64-unknown-linux,{x86_64,i686}-pc-windows}-gnu; do
  echo "${target}"
  rustup target add "${target}" || continue
  cross build --bin astrolabe --release --target "${target}" >/dev/null 2>&1 && echo "${target} ✅" || echo "${target} ❌"
done

# Build and pack wasm files
wasm-pack build --target web --release -d "../target/wasm/pkg" wasm &&
  tar -czvf "target/wasm/wasm-files.tar.gz" "target/wasm/pkg" &&
  echo rm -rf "target/wasm/pkg"

mkdir -p release/{completions,manpage}

move_bin() {
  mv target/$1/release/astrolabe release/astrolabe-$2
}

# Pack files to a common folder
#move_bin arm-unknown-linux-gnueabihf rasp-pi
move_bin x86_64-unknown-linux-gnu x64
#move_bin i686-unknown-linux-gnu x32
move_bin x86_64-pc-windows-gnu x64.exe
move_bin i686-pc-windows-gnu x32.exe

mv target/x86_64-unknown-linux-gnu/release/build/astrolabe-bin-*/out/* release/completions
mv release/completions/astrolabe.1 release/manpage
