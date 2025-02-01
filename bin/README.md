# Astrolabe binary

[![License](https://img.shields.io/github/license/2sugarcubes/astrolabe)](https://github.com/2sugarcubes/astrolabe/LICENSE.txt)
[![Code Coverage](https://codecov.io/gh/2sugarcubes/astrolabe/branch/dev/graph/badge.svg?token=E27GPTMWQY)](https://codecov.io/github/2sugarcubes/astrolabe)
[![GitHub Release Workflow Status](https://img.shields.io/github/actions/workflow/status/2sugarcubes/astrolabe/release.yml)](https://github.com/2sugarcubes/astrolabe/releases)
[![GitHub Test Workflow Status](https://img.shields.io/github/actions/workflow/status/2sugarcubes/astrolabe/tests.yml?label=tests)](https://github.com/2sugarcubes/astrolabe/actions/workflows/tests.yml)
![Total commits](https://img.shields.io/github/commit-activity/t/2sugarcubes/astrolabe/dev)
[![Open Issues](https://img.shields.io/github/issues/2sugarcubes/astrolabe)](https://github.com/2sugarcubes/astrolabe/issues)

A binary that assists with generating star-charts for arbitrary universes
or solar systems.

## Getting started

You can either [compile this binary yourself](https://github.com/2sugarcubes/astrolabe/tree/master/bin/README.md#compiling)
or use a [pre-compiled binary](https://github.com/2sugarcubes/astrolabe/releases)

### Building

1. Ensure you have `cargo` [installed](https://doc.rust-lang.org/cargo/getting-started/installation.html)
2. If you want to run the binary through cargo run `cargo run -- {your arguments}`
3. otherwise run `cargo build --release` or `cargo install`

   - You will see some warnings like this `warning: astrolabe-bin@{version}:
Completion file for {shell} has been generated at: "/path/to/astrolabe/
target/release/build/astrolabe-bin-{hash}/..."`
   - If you want completions for your shell append the appropriate file
     to your completions file.

     - e.g. for bash
       `cat target/release/build/astrolabe-bin-*/astrolabe.bash >> ~/.bash_completion`
     - e.g. for zsh
       `cp target/release/build/astrolabe-bin-*/_astrolabe ~/.local/lib/oh-my-zsh/cache/completions/`
     - e.g. for powershell
       `type target\release\build\astrolabe-bin-*\_astrolabe.ps1 >> $profile`

   - You can also install the man page by running
     `sudo cp target/release/build/astrolabe-bin-*/out/astrolabe.1 /usr/local/share/man/man1/`

4. if you ran cargo `cargo build` copy the compiled binary where you would like it
   (e.g. into your `$path`, which is what `cargo install` does)
   - The compiled binary will be at `target/release/astrolabe`

### Running

Now that the binary is installed you can run
`astrolabe --output universe.json build --star-count 1000` to generate a
thousand stars. You can then download [this file](https://raw.githubusercontent.com/2sugarcubes/astrolabe/refs/heads/dev/assets/test/generated/observatories.json)
since we do not automatically generate observatories yet.

Then you can generate observations with this command `astrolabe simulate
-end-time 5 --universe universe.json --observatories observatories.json`
to generate observations from all observatories for times 0, 1, 2, 3, and 4
