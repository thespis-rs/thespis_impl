# thespis_impl

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)
[![Build Status](https://api.travis-ci.org/najamelan/thespis_impl.svg?branch=master)](https://travis-ci.org/najamelan/thespis_impl)
[![Docs](https://docs.rs/thespis_impl/badge.svg)](https://docs.rs/thespis_impl)
[![crates.io](https://img.shields.io/crates/v/thespis_impl.svg)](https://crates.io/crates/thespis_impl)


> The reference implementation of the thespis actor model

This crate implements the traits from the `thespis` crate. It provides what you need to use actors.



## Table of Contents

- [Install](#install)
   - [Upgrade](#upgrade)
   - [Dependencies](#dependencies)
   - [Security](#security)
- [Usage](#usage)
   - [Basic Example](#basic-example)
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

  thespis_impl: ^0.1
```

In Cargo.toml:
```toml
[dependencies]

  thespis_impl = "0.1"
```

### Upgrade

Please check out the [changelog](https://github.com/thespis-rs/thespis_impl/blob/master/CHANGELOG.md) when upgrading.


### Dependencies

This crate has few dependencies. Cargo will automatically handle it's dependencies for you.

There are no optional features.


### Security




## Usage



### Basic example

```rust

```

## API

API documentation can be found on [docs.rs](https://docs.rs/thespis_impl).


## Contributing

Please check out the [contribution guidelines](https://github.com/thespis-rs/thespis_impl/blob/master/CONTRIBUTING.md).


### Testing


### Code of conduct

Any of the behaviors described in [point 4 "Unacceptable Behavior" of the Citizens Code of Conduct](https://github.com/stumpsyn/policies/blob/master/citizen_code_of_conduct.md#4-unacceptable-behavior) are not welcome here and might get you banned. If anyone, including maintainers and moderators of the project, fail to respect these/your limits, you are entitled to call them out.

## License

[Unlicence](https://unlicense.org/)




## TODO


### Tests
- tests for processing messages concurrently when the future of handle doesn't need to access state.

- test supervision + error message logged when actor panics (does it have actor name in it?).
  - look at other api's for supervision, akka, erlang.

- flesh out tests and comment what's being tested. Consistency, expect or return result.

- test ringchannel


## API

- Isolating mutable state and enforcing immutable messages guarantees implicit synchronization. However, the concept of asynchronous messaging and no global state challenges coordination. An application may require consensus or a concerted view of state between multiple actors. When multiple actors must be strictly orchestrated in order to provide a distinct application function, correct messaging can become very demanding. Thus, many implementations provide higher-level abstractions that implement low-level coordination protocols based on complex message flows, but hide the internal complexity from the developer. For Erlang, OTP is a standard library that contains a rich set of abstractions, generic protocol implementations and behaviors.

- Another common approach is the transactor. For example, multiple actors may require to modify their internal state in a coordinated manner. A transactor, which is a dedicated actor for coordinating transactional operations of multiple actors, can help in this situation by providing abstract transaction logic. Some transactors also apply STM concepts for transactional behavior [Les09,Les11].


- use ArcStr?
- https://berb.github.io/diploma-thesis/original/054_actors.html
- inbox, cooperative yielding every x messages?
- move clonesink to chanx
- contention benchmark, actix is faster...
- in thespis_remote we get one more DROP address log than CREATE. We should get to the bottom of this phantom address as it throws of debugging.
- document the fact that we use catch_unwind

## Types of channels:

- priority channels, ...
- futures
- crossbeam
- ring_channel
- ringbuf with pointers
- futures-intrusive
