# Examples

These examples demonstrate how to use thespis with the reference implementation (thespis_impl).

1. *basic*: the most simple example
2. *desugar*: desugar the `Addr::builder` by creating everything manually.
3. *across_yields*: showing the use of &mut self across await points in handle trait method
4. *perf*: an example I use with cargo flamegraph to check where time is spent
5. *recipient*: Store addresses to several actors that accept the same message type
6. *recipient_is_sink*: Shows how to you can use sink combinators on recipient
7. *multi_thread*: Use addresses to send across threads
8. *move_fut*: Use address on the same thread, but move the future from call to a different thread before polling it.
9. *local_spawn*: Use an Actor which is `!Send` and spawn it on a thread local executor.
10. *concurrent*: Let an actor process messages concurrently when no mutable state is needed.
11. *concurrent_nursery*: Let an actor process messages concurrently when no mutable state is needed and process the results of the operation in the actor rather than returning them to the caller.
12. *drop_channel*: An example of using a channel that overwrites older messages instead of providing back pressure.

TODO: Receiver as Box<Any>
