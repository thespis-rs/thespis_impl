# thespis - CHANGELOG

## 0.1.0-alpha.3 - 2020-11-17

  - Add `WeakAddr`, which will not keep the mailbox alive if all `Addr` are dropped.
  - When the mailbox closes normally, it now returns the actor from the `JoinHandle`.
  - FIX: tracing log output is now correct and has spans identifying the actor for all messages from Addr and Mailbox.
    Spans also mention the type of the actor.
  - Removed tokio channels. This means we don't have to depend on async_chanx. Users can still use tokio channels
    if they have a Sink implementation for the sender.
  - to simplify the API, the name for an actor is now taken as `Option<&str>` rather than `Option<Arc<str>>`.
  - `Receiver` was removed as you can actually downcast `Box< dyn Any >` to `Box< dyn Address<_>`, so there shouldn't
    be a need for `Receiver`.
  - Move CI to github.
  - Add tests on Wasm.

## 0.1.0-alpha.2 - 2020-11-1

	- update dependencies.
	- fix docs.rs

## 0.1.0-alpha.1 - 2020-09-14

	- initial release, not for production.




