# Examples

These examples demonstrate how to use thespis with the reference implementation (thespis_impl).

1. *basic*: the most simple example
2. *across_yields*: showing the use of &mut self across await points in handle trait method
3. *no_rt*: use without the convenience of the runtime, but with full control over mailbox
4. *perf*: an example I use with cargo flamegraph to check where time is spent
5. *recipient*: Store addresses to several actors that accept the same message type
6. *recipient_is_sink*: Shows how to you can use sink combinators on recipient
7. *multi_thread*: Use addresses to send across threads
8. *move_fut*: Use address on the same thread, but move the future from call to a different thread before polling it.
9. *local_spawn*: Use an Actor which is `!Send` and spawn it on a thread local executor.

TODO: Receiver as Box<Any>
