# Examples

These examples demonstrate how to use thespis with the reference implementation (thespis_impl).

1. [*basic*](/basic.rs): the most simple example
2. [*desugar*](/desugar.rs): desugar the `Addr::builder` by creating everything manually.
3. [*across_yields*](/across_yields.rs): showing the use of &mut self across await points in handle trait method.
4. [*perf*](/perf): examples I use with cargo flamegraph or profiler.firefox.com to check where time is spent.
5. [*recipient*](/recipient.rs): Store addresses to several actors that accept the same message type.
5. [*recipient_any*](/recipient_any.rs): Store addresses to several actors that accept different message types.
6. [*addr_is_sink*](/addr_is_sink.rs): Shows how to you can use sink combinators on recipient
8. [*move_fut*](/move_fut.rs): Use address on the same thread, but move the future from call to a different thread before polling it.
9. [*local_spawn*](/local_spawn.rs): Use an Actor which is `!Send` and spawn it on a thread local executor.
10. [*concurrent*](/concurrent.rs): Let an actor process messages concurrently when no mutable state is needed.
11. [*concurrent_nursery*](/concurrent_nursery.rs): Let an actor process messages concurrently when no mutable state is needed. This time we make sure that none of the spawned subtasks can outlive our actor.
12. [*drop_channel*](/drop_channel.rs): An example of using a channel that overwrites older messages instead of providing back pressure.
13. [*supervisor*](/supervisor.rs): How to supervise an actor in case it panics.
14. [*tokio_channel*](/tokio_channel): How to use a tokio channel with thespis.
15. [*deadlock_prio*](../tests/deadlock.rs): Use a priority channel with thespis to give an actor a double mailbox and avoid a deadlock in a specific situation. Note you can also use priority channels like this for other usecases, eg. you can make a channel that will poll different addresses in an alternating pattern.
