# thespis_impl

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)
[![Build Status](https://api.travis-ci.org/najamelan/thespis_impl.svg?branch=master)](https://travis-ci.org/najamelan/thespis_impl)
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

  thespis_impl: ^0.1-alpha
```

In Cargo.toml:
```toml
[dependencies]

  thespis_impl = "0.1-alpha"
```

### Upgrade

Please check out the [changelog](https://github.com/thespis-rs/thespis_impl/blob/master/CHANGELOG.md) when upgrading.


### Dependencies

This crate has few dependencies. Cargo will automatically handle it's dependencies for you.

There is one optional feature: `tokio_channel`. This causes the `ActorBuilder` to use tokio channels by default instead of futures channels.


### Security

This crate has `#![forbid(unsafe_code)]`, but our dependencies do use unsafe.


## Usage

Please check out the [guide level documentation](https://thespis-rs.github.io/thespis_guide/) and the [examples in the repository](https://github.com/thespis-rs/thespis_impl/blob/master/examples).

## API

API documentation can be found on [docs.rs](https://docs.rs/thespis_impl).


## Contributing

Please check out the [contribution guidelines](https://github.com/thespis-rs/thespis_impl/blob/master/CONTRIBUTING.md).


### Testing

`cargo test --all-features`.


### Code of conduct

Any of the behaviors described in [point 4 "Unacceptable Behavior" of the Citizens Code of Conduct](https://github.com/stumpsyn/policies/blob/master/citizen_code_of_conduct.md#4-unacceptable-behavior) are not welcome here and might get you banned. If anyone, including maintainers and moderators of the project, fail to respect these/your limits, you are entitled to call them out.

## License

[Unlicence](https://unlicense.org/)
