# To-do lists

## Before each merge

- [ ] Add documentation
- [ ] Add tests
- [ ] Cleanup clippy complaints
- [ ] Cleanup [cargo.toml]

## Known Bugs

- [ ] Program hangs when Artifexian generator is run with the mock RNG and
      more than seven consecutive bits are 0

## Non-functional requirements

- [ ] Tweak Artifexian generator (or generators in general) to only generate
      stars in a sphere around the observing body to make the sky more densely
      populated while also less computationally demanding
- [ ] Make compile time smaller by making wasm-bindgen inclusion dependant on
      target arch
- [ ] Make serde an optional dependency to reduce compile time and binary size
