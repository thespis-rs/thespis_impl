# thespis_impl
The reference implementation of the thespis actor model


## TODO

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
