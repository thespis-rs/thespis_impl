# thespis_impl - CHANGELOG

## [Unreleased]

[Unreleased]: https://github.com/najamelan/async_executors/compare/release...dev


## [0.1.0] - 2021-06-20

[0.2.0]: https://github.com/najamelan/async_executors/compare/0.1.0...0.2.0

  - __BREAKING__: Update async_executors and async_nursery.
  - Change ActorBuilder::name to take an impl `AsRef<str>`. This should be more flexible than before so it shouldn't be breaking. 


## [0.1.0] - 2021-06-20

[0.1.0]: https://github.com/najamelan/async_executors/compare/0.1.0-alpha.3...0.1.0

### Upgraded

  - Depend on _thespis_ 0.1.

### Fixed

  - Improve error messages and logging.
  - switch to `oneshot` crate for better performance.

### Added

  - `ActorInfo`: a type that represents information about an actor like name, id and spans for tracing.
  - `src` property on `ThesErr` and implement `Error::source`, so we return error causes.

### Removed

  - __BREAKING__: The spawning function from `Mailbox`. Now you just have `start_` functions which return a future. The `ActorBuilder`
    still has the spawn functions for convenience. They used to be called `start`, but are now renamed to `spawn`.


## [0.1.0-alpha.3] - 2021-05-28

[0.1.0-alpha.3]: https://github.com/najamelan/async_executors/compare/0.1.0-alpha.2...0.1.0-alpha.3

### Added
  - Add `WeakAddr`, which will not keep the mailbox alive if all `Addr` are dropped.
  - When the mailbox closes normally, it now returns the actor from the `JoinHandle`.

### Fixed
  - tracing log output is now correct and has spans identifying the actor for all messages from Addr and Mailbox.
    Spans also mention the type of the actor.
  - __BREAKING__: to simplify the API, the name for an actor is now taken as `Option<&str>` rather than `Option<Arc<str>>`.
  - Move CI to github.
  - Add tests on Wasm.

### Removed
  - __BREAKING__: Removed tokio channels. This means we don't have to depend on async_chanx. Users can still use tokio channels
    if they have a Sink implementation for the sender.
  - __BREAKING__:`Receiver` was removed as you can actually downcast `Box< dyn Any >` to `Box< dyn Address<_>`, so there shouldn't
    be a need for `Receiver`.


## 0.1.0-alpha.2 - 2020-11-1

	- update dependencies.
	- fix docs.rs

## 0.1.0-alpha.1 - 2020-09-14

	- initial release, not for production.




