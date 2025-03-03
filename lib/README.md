# Astrograph

[![License](https://img.shields.io/github/license/2sugarcubes/astrograph)](https://github.com/2sugarcubes/astrograph/LICENSE.txt)
[![Code Coverage](https://codecov.io/gh/2sugarcubes/astrograph/branch/dev/graph/badge.svg?token=E27GPTMWQY)](https://codecov.io/github/2sugarcubes/astrograph)
[![GitHub Release Workflow Status](https://img.shields.io/github/actions/workflow/status/2sugarcubes/astrograph/release.yml)](https://github.com/2sugarcubes/astrograph/releases)
[![GitHub tests status](https://img.shields.io/github/check-runs/2sugarcubes/astrograph/dev)](https://github.com/2sugarcubes/astrograph/actions)
![Total commits](https://img.shields.io/github/commit-activity/t/2sugarcubes/astrograph/dev)
[![Open Issues](https://img.shields.io/github/issues/2sugarcubes/astrograph)](https://github.com/2sugarcubes/astrograph/issues)
[![PRs Welcome](https://img.shields.io/badge/PRs-Welcome-leaf--green)](https://makeapullrequest.com)
[![First-timers-only friendly](https://img.shields.io/badge/first--timers--only-friendly-leaf--green)](https://www.firsttimersonly.com/)

A library for predicting the locations of astronomical bodies that are both deterministic and non-chaotic.

## What this package aims to do

Provide a simple way of generating star maps for world-builders and writers, by allowing for varying levels of granularity in your simulations.

Allow users to generate astronomical tables that may be useful for e.g. Game masters wanting to introduce [astrology](https://en.wikipedia.org/wiki/Astrology)/[astromancy](https://en.wikipedia.org/wiki/Astromancy) elements into their games.

## What this package will not do

Predict bodies in a n-body problem, a situation where each body influences the motion of every other body. This is largely done to enable querying an arbitrary time without needing to querying every time before it, and allow querying times before the epoch.

## Feature roadmap

- [x] Fixed bodies, useful for roots or bodies with very long orbital periods e.g. distant galaxies
- [x] Keplerian bodies
- [x] Bodies, defines how dynamics relate to one another in parent/children relationships
- [x] Rotating bodies, will be useful for observatories on bodies, possibly for drawing scenes later as well
- [x] Observatories, define the latitude, longitude, and altitude of the observer for observation times
- [x] Different projections, default will be an orthographic projection, but other projections will likely be added on a low priority
- [x] Writing to file, probably an SVG, but possibly PNG/BMP/etc if I see a need for it.
- [x] Configurable precision, i.e. [F32](https://en.wikipedia.org/wiki/Single-precision_floating-point_format), [F64](https://en.wikipedia.org/wiki/Double-precision_floating-point_format), and possibly [F128](https://en.wikipedia.org/wiki/Quadruple-precision_floating-point_format)
- [x] Serialization, most likely json, but other [supported serialization data formats](https://serde.rs/#data-formats) will be added on an as needed basis
- [x] Procedurally generated universes
  - [x] Arbitrary elliptic planes around stars
  - [x] Arbitrary elliptic planes around planets
- [x] WASM target
- [x] binary target
- [x] Eclipses
- [ ] web page
- [ ] Summary output format (Similar to an [almanac](https://en.wikipedia.org/wiki/Almanac) e.g. [Astronomical Almanac, 2016, by The Stationery Office, U.S. Nautical Almanac Office, and Defense Department](https://openlibrary.org/books/OL50688251M/Astronomical_Almanac))
- [ ] Constellations
- [ ] Color coded/labeled Bodies
- [ ] Body classes (e.g. `planet-rocky`, `planet-gas`, `star-M-class`, `moon-icy`, `black-hole`), useful for filtering bodies in results
