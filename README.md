# thespis_impl
The reference implementation of the thespis actor model


## TODO

- in thespis_remote we get one more DROP address log than CREATE. We should get to the bottom of this phantom address as it throws of debugging.
- get rid of the need to spawn in Addr, just return the future always, remove the dependency on async_runtime
- clean up benches
- use generic-channel crate? allow ring-channel, bounded channel?
- switch to tracing. for logging and metrics.

## Types of channels:

- futures
- crossbeam
- ring_channel
- ringbuf with pointers
- futures-intrusive


