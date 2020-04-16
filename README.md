# thespis_impl
The reference implementation of the thespis actor model


## TODO

- !Send Messages.

- test supervision + error message logged when actor panics (does it have actor name in it?).
- document the fact that we use catch_unwind

- supervisor type? take a closure that produces the actor?

- Isolating mutable state and enforcing immutable messages guarantees implicit synchronization. However, the concept of asynchronous messaging and no global state challenges coordination. An application may require consensus or a concerted view of state between multiple actors. When multiple actors must be strictly orchestrated in order to provide a distinct application function, correct messaging can become very demanding. Thus, many implementations provide higher-level abstractions that implement low-level coordination protocols based on complex message flows, but hide the internal complexity from the developer. For Erlang, OTP is a standard library that contains a rich set of abstractions, generic protocol implementations and behaviors.

- Another common approach is the transactor. For example, multiple actors may require to modify their internal state in a coordinated manner. A transactor, which is a dedicated actor for coordinating transactional operations of multiple actors, can help in this situation by providing abstract transaction logic. Some transactors also apply STM concepts for transactional behavior [Les09,Les11].

- supervisor support in inbox. Take an address that can receive ActorCrashed message with type and id information.
  - catch_unwind? Given that actors are not supposed to share state... should be ok
  - pharos on inbox for events? Maybe not worth it if ActorCrashed is the only message, but could be for back pressure too.
- back pressure when sending to self? Might deadlock.
- let an actor decide to ignore certain messages
- https://berb.github.io/diploma-thesis/original/054_actors.html
- inbox, cooperative yielding every x messages?
- Sync issues on channel errors argh...
- move clonesink to chanx
- contention benchmark, actix is faster...
- flesh out tests and comment what's being tested. Consistency, expect or return result.
- drop channel, priority channels, ...
- in thespis_remote we get one more DROP address log than CREATE. We should get to the bottom of this phantom address as it throws of debugging.
- clean up benches
- switch to tracing. for logging and metrics.

## Types of channels:

- futures
- crossbeam
- ring_channel
- ringbuf with pointers
- futures-intrusive
