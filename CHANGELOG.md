# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - ReleaseDate
### Changed
* Made logging level configurable
* Help text in GUI

## [1.0.0-alpha.1] - 2020-02-06
### Changed
* Updated dependencies (cargo update)
* Switched to the new `Cargo.lock` format
### Fixed
* Some clippy lints

## [0.5.2] - 2019-09-12
### Changed
* Switched to stable Rust

## [0.5.1] - 2019-04-15
### Added
* HiDPI support

## [0.5.0] - 2019-03-23
### Added
* Tracing depletion-mode FETs
### Changed
* Faster trace capture
### Fixed
* Panic when trying to fit a model to an empty trace
* Bias level for BJT devices

## [0.4.2] - 2019-03-19
### Added
* Spinner animation during capture, load and model fitting #24
### Changed
* Capturing and loading a trace is now async #24
* Tidied up BJT and FET device parameters panes

## [0.4.1] - 2019-03-16
### Changed
* Tweaked the right pane a little bit
* Panning the plot only when tracing, not when selecting another type of DUT
### Fixed
* Panic when there's no AD2 connected

## [0.4.0] - 2019-03-15
### Added
* Adjusting DUT parameters, such as BJT/FET bias levels
### Changed
* Refactored the device types, made traces their associated types and encapsulated AOIs

## [0.3.0] - 2019-03-09
### Added
* Tracing FETs #4
### Changed
* Improved installation guide in `README.md`
* Better text rendering on the graph
* Caching scatter plots #23

## [0.2.1] - 2019-03-07
### Added
* Reading and writing `*.csv.gz` files. For now you'd have to manually specify the `.gz` extension in the file save dialog.
### Changed
* Refactored the hierarchy of traces and models
### Fixed
* Removed many `.unwrap()` calls

## [0.2.0] - 2019-03-06
### Added
* Tracing BJTs #3
* Connection hints and legend panes #15

## [0.1.11] - 2019-03-04
### Added
* Benchmarks
* Some sample diode data
* Save and Load buttons in GUI
### Changed
* Better logging in the curve fitting module
### Fixed
* 1N4728A model fitting

## [0.1.10] - 2019-03-03
### Changed
* Extracted and generified the Gauss-Newton algorithm
* Adjusted model fitting parameters for more precision

## [0.1.9] - 2019-03-01
### Added
* Adjustable zoom level #22
### Changed
* Better right pane layout

## [0.1.8] - 2019-02-28
### Added
* Console logging
### Changed
* Heavily improved model fitting
### Fixed
* Engineering notation NaN and Inf handling

## [0.1.7] - 2019-02-25
### Added
* Showing backtrace on errors
### Changed
* Triggering AWG on scope capture #1
### Removed
* Removed armv7 builds for the time being #21

## [0.1.6] - 2019-02-24
### Changed
* Tweak cargo release #21

## [0.1.5] - 2019-02-24
### Changed
* Tweak cargo release #21

## [0.1.4] - 2019-02-24
### Added
* Building .deb packages #18
* Building .rpm (#19), arch (#20) packages
* Initial versions of flatpak (#16) and snap (#17) manifests

## [0.1.3] - 2019-02-21
### Added
* Window and .exe icon #7
### Changed
* Sensible initial window size #14

## [0.1.2] - 2019-02-21
### Added
* Proper Shockley diode model #5

## [0.1.1] - 2019-02-18
### Changed
* faster CI builds
* better CI release uploads

## [0.1] - 2019-02-18
### Initial release
* Can trace diodes
* Fitting a simplified Shockley diode model to a trace
