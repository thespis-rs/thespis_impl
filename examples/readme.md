# Examples

These examples demonstrate how to use thespis with the reference implementation (thespis_impl).

1. *basic*: the most simple example
2. *across_yields*: showing the use of &mut self across await points in handle trait method
3. *perf*: an example I use with cargo flamegraph to check where time is spent
4. *recipient*: Store addresses to several actors that accept the same message type
5. *recipient_is_sink*: Shows how to you can use sink combinators on recipient
6. *multi_thread*: Use addresses to send across threads
7. *move*: Use address on the same thread, but move the future from call to a different thread before polling it.
8. *local_spawn*: Use an Actor which is `!Send` and spawn it on a thread local executor.
9. *concurrent*: Let an actor process messages concurrently when no mutable state is needed.

TODO: Receiver as Box<Any>
