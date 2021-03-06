# Curve tracer GUI for AD2 curve tracer

[![ci](https://travis-ci.org/knack-supply/curve-tracer.svg?branch=master)](https://travis-ci.org/knack-supply/curve-tracer)
[![win ci](https://ci.appveyor.com/api/projects/status/g7ov8xujfwa11rg7/branch/master?svg=true)](https://ci.appveyor.com/project/ilya-epifanov/curve-tracer/branch/master)

A companion app for [AD2 curve tracer](https://knack.supply/product/ad2ct/)

## Installation

### Linux

`.deb`'s are available on [release downloads page].

To use a plain binary release, install the dependencies first (libgtk3, libcairo and 64-bit [Digilent Waveforms]). 
Then download and run `ks-curve-tracer-linux-amd64` binary from [release downloads page].

### Windows

Install GTK3 runtime first. Grab a latest version from https://github.com/tschoonj/GTK-for-Windows-Runtime-Environment-Installer/releases, e.g. https://github.com/tschoonj/GTK-for-Windows-Runtime-Environment-Installer/releases/download/2019-02-07/gtk3-runtime-3.24.4-2019-02-07-ts-win64.exe.
64-bit [Digilent Waveforms] has to be installed, too.

Download and run a `ks-curve-tracer.exe` from [release downloads page].

### Mac OS

TBD, not tested yet.

`brew install gtk+3 cairo atk`

Download and run a `ks-curve-tracer-macos` from [release downloads page].

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
 * MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

[Digilent Waveforms]: https://reference.digilentinc.com/reference/software/waveforms/waveforms-3/start
[release downloads page]: https://github.com/knack-supply/curve-tracer/releases