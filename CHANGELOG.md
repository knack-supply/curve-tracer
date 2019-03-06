# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
