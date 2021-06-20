## TODO

fix code coverage, currently still on travis.

- verify log spans.
   - see tests/tracing.rs See: https://github.com/dbrgn/tracing-test/issues/4
   - neither wasm-logger, nor tracing-wasm show the current span atm. See: https://github.com/storyai/tracing-wasm/issues/17

### Tests

- flesh out tests.

# Perf

- rerun and analyze benchmarks
- comparison to actix, how do they spread the load better across threads?


## API

- Can we let the user recover their message from the error if sending fails? Is more relevant now since we
  have the WeakAddr. -> Should be possible if we change call to poll_call. We can only return the message
  if the error is in poll_ready of the underlying sink. It won't work for errors in start_send and poll_flush.
  -> For the moment there is no good solution. Either:
     1. SinkExt::send is changed to return the message,
     2. or GAT's so we can have have Call as associated type and let it capture the lifetime of self,
     3. or we break a lot of the design that splits interface from implementation and we have to implement
        Call in thespis, which implies having the oneshot for the return type, having Envelope etc.

- Isolating mutable state and enforcing immutable messages guarantees implicit synchronization. However, the concept of asynchronous messaging and no global state challenges coordination. An application may require consensus or a concerted view of state between multiple actors. When multiple actors must be strictly orchestrated in order to provide a distinct application function, correct messaging can become very demanding. Thus, many implementations provide higher-level abstractions that implement low-level coordination protocols based on complex message flows, but hide the internal complexity from the developer. For Erlang, OTP is a standard library that contains a rich set of abstractions, generic protocol implementations and behaviors.

- Another common approach is the transactor. For example, multiple actors may require to modify their internal state in a coordinated manner. A transactor, which is a dedicated actor for coordinating transactional operations of multiple actors, can help in this situation by providing abstract transaction logic. Some transactors also apply STM concepts for transactional behavior [Les09,Les11].


- https://berb.github.io/diploma-thesis/original/054_actors.html


## Types of channels:

- priority channels, ...
- futures
- crossbeam
- ring_channel
- ringbuf with pointers
- futures-intrusive
