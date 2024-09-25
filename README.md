# Astrolabe

[![License](https://img.shields.io/github/license/2sugarcubes/astrolabe)](https://github.com/2sugarcubes/astrolabe/LICENSE.txt)

[![codecov](https://codecov.io/github/2sugarcubes/astrolabe/graph/badge.svg?token=E27GPTMWQY)](https://codecov.io/github/2sugarcubes/astrolabe)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/2sugarcubes/astrolabe/tests)](https://github.com/2sugarcubes/astrolabe/actions)
[![Total commits](https://img.shields.io/github/commit-activity/t/2sugarcubes/astrolabe)]()

[![Open Issues](https://img.shields.io/github/issues/2sugarcubes/astrolabe)](https://github.com/2sugarcubes/astrolabe/issues)

A library for predicting the locations of astronomical bodies that are both deterministinc and non-chaotic.

## What this package aims to do

Provide a simple way of generating star maps for world-builders and writers, by allowing for varying levels of granularity in your simulations.

Allow users to generate astronomical tables that may be useful for e.g. Game masters wanting to introduce [astrology](https://en.wikipedia.org/wiki/Astrology)/[astromancy](https://en.wikipedia.org/wiki/Astromancy) elements into their games.

## What this package will not do

Predict bodies in a n-body problem, a situation where each body influences the motion of every other body. This is largely done to enable querying an arbitrary time without needing to querying every time before it, and allow querying times before the epoch.

## Feature roadmap

- [x] Fixed bodies, useful for roots or bodies with very long orbital periods e.g. distant galaxies
- [x] Keplerian bodies
- [ ] Bodies, defines how dynamics relate to one another in parent/children relationships
- [ ] Rotating bodies, will be useful for observatories on bodies, possibly for drawing scenes later as well
- [ ] Observatories, define the latitude, longitude, and altitude of the observer for observation times
- [ ] Different projections, default will be an orthographic projection, but other projections will likely be added on a low priority
- [ ] Writing to file, probably an SVG, but possibly PNG/BMP/etc if I see a need for it.
- [ ] Configurable precision, i.e. [F32](https://en.wikipedia.org/wiki/Quadruple-precision_floating-point_format), [F64](https://en.wikipedia.org/wiki/Double-precision_floating-point_format), and possibly [F128](https://en.wikipedia.org/wiki/Quadruple-precision_floating-point_format)
- [ ] Serialisation, most likely json, but other [supported serialisation data formats](https://serde.rs/#data-formats) will be added on an as needed basis
- [ ] Procedurally generated universes
- [ ] Color coded Bodies
- [ ] Body classes (e.g. `planet-rocky`, `planet-gas`, `star-M-class`, `moon-icy`, `black-hole`), useful for filtering bodies in results
- [ ] Constelations
- [ ] WASM target
- [ ] web page
