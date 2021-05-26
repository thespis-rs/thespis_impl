## TODO

- verify log spans.
   - see tests/tracing.rs
   - In wasm-logger the spawns show up as error and it spams opening and closing of spans.
   - guide level docs about debugging.

- verify the need for receiver. You can actually downcast Box<dyn Any> to Box<dyn Address<M>>. So we might not have to wrap it in a struct.
- can we rate limit actor messages with stream_throttle?

### Tests
- switch to github actions and disable travis.

- tests for processing messages concurrently when the future of handle doesn't need to access state.
- wasm tests

- flesh out tests and comment what's being tested. Consistency, expect or return result.

- test ringchannel

# Perf

- async-oneshot claims to be very fast
- rerun and analyze benchmarks
- comparison to actix, how do they spread the load better across threads?


## API

- Can we let the user recover their message from the error if sending fails? Is more relevant now since we
  have the WeakAddr.

- Isolating mutable state and enforcing immutable messages guarantees implicit synchronization. However, the concept of asynchronous messaging and no global state challenges coordination. An application may require consensus or a concerted view of state between multiple actors. When multiple actors must be strictly orchestrated in order to provide a distinct application function, correct messaging can become very demanding. Thus, many implementations provide higher-level abstractions that implement low-level coordination protocols based on complex message flows, but hide the internal complexity from the developer. For Erlang, OTP is a standard library that contains a rich set of abstractions, generic protocol implementations and behaviors.

- Another common approach is the transactor. For example, multiple actors may require to modify their internal state in a coordinated manner. A transactor, which is a dedicated actor for coordinating transactional operations of multiple actors, can help in this situation by providing abstract transaction logic. Some transactors also apply STM concepts for transactional behavior [Les09,Les11].


- https://berb.github.io/diploma-thesis/original/054_actors.html
- contention benchmark, actix is faster...


## Types of channels:

- priority channels, ...
- futures
- crossbeam
- ring_channel
- ringbuf with pointers
- futures-intrusive
