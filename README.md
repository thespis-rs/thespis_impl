# thespis_impl
The reference implementation of the thespis actor model


## TODO

- get rid of failure
- get rid of the need to spawn in Addr, just return the future always, remove the dependency on async_runtime
- clean up benches
- use generic-channel crate? allow ring-channel, bounded channel?
- use crossbeam channels? in pharos as well?
- test runtime with threads

## Types of channels:

- futures
- crossbeam
- ring_channel
- ringbuf with pointers
- futures-intrusive


