# thespis_impl

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)
[![Build Status](https://api.travis-ci.org/najamelan/thespis_impl.svg?branch=release)](https://travis-ci.org/najamelan/thespis_impl)
[![Docs](https://docs.rs/thespis_impl/badge.svg)](https://docs.rs/thespis_impl)
[![crates.io](https://img.shields.io/crates/v/thespis_impl.svg)](https://crates.io/crates/thespis_impl)


> The reference implementation of the thespis actor model

This crate implements the traits from the `thespis` crate. It provides what you need to use actors.

Please check out the [guide level documentation](https://thespis-rs.github.io/thespis_guide/).


## Table of Contents

- [Install](#install)
   - [Upgrade](#upgrade)
   - [Dependencies](#dependencies)
   - [Security](#security)
- [Usage](#usage)
   - [API](#api)
- [Contributing](#contributing)
   - [Code of Conduct](#code-of-conduct)
- [License](#license)


## Install
With [cargo add](https://github.com/killercup/cargo-edit):
`cargo add thespis_impl`

With [cargo yaml](https://gitlab.com/storedbox/cargo-yaml):
```yaml
dependencies:

  thespis_impl: ^0.2
```

In Cargo.toml:
```toml
[dependencies]

  thespis_impl = "0.2"
```

### Upgrade

Please check out the [changelog](https://github.com/thespis-rs/thespis_impl/blob/release/CHANGELOG.md) when upgrading.


### Dependencies

This crate has few dependencies. Cargo will automatically handle it's dependencies for you.


### Security

It is recommended to always use [cargo-crev](https://github.com/crev-dev/cargo-crev) to verify the trustworthiness of each of your dependencies, including this one. 

This crate has `#![forbid(unsafe_code)]`, but our dependencies do use unsafe.


## Usage

Please check out the [guide level documentation](https://thespis-rs.github.io/thespis_guide/) and the [examples in the repository](https://github.com/thespis-rs/thespis_impl/blob/release/examples).

## API

API documentation can be found on [docs.rs](https://docs.rs/thespis_impl).


## Contributing

Please check out the [contribution guidelines](https://github.com/thespis-rs/thespis_impl/blob/release/CONTRIBUTING.md).


### Testing

`cargo test --all-features`.


### Code of conduct

Any of the behaviors described in [point 4 "Unacceptable Behavior" of the Citizens Code of Conduct](https://github.com/stumpsyn/policies/blob/release/citizen_code_of_conduct.md#4-unacceptable-behavior) are not welcome here and might get you banned. If anyone, including maintainers and moderators of the project, fail to respect these/your limits, you are entitled to call them out.

## License

[Unlicence](https://unlicense.org/)
