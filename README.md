# thespis_impl
The reference implementation of the thespis actor model


## TODO

- The runtime convience needs work. Currently we use RefCell, but don't return the error of a theoretical double borrow to the user. In that case we have runtime crashes + the overhead of RefCell with no benefit... It probably makes sense to use unsafe here to avoid the overhead if we can guarantee that it will never crash.
- clean up benches
- use generic-channel crate? allow ring-channel, bounded channel?
- use crossbeam channels? in pharos as well?
- test runtime with threads

