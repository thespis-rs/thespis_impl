# Examples

These examples demonstrate how to use thespis with the reference implementation (thespis_impl).

1. *basic*: the most simple example
2. *across_yields*: showing the use of &mut self across await points in handle trait method
3. *no_rt*: use without the convenience of the runtime, but with full control over mailbox
4. *pers*: an example I use with flamegraph to check where time is spent
5. *recipient*: Store addresses to several actors that accept the same message type
6. *multi_thread*: Use addresses to send across threads
7. *move_fut*: Use address on the same thread, but move the future from call to a different thread before polling it.
