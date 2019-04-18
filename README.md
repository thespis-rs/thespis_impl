# thespis_impl
The reference implementation of the thespis actor model


## TODO

The runtime convience needs work. Currently we use RefCell, but don't return the error of a theoretical double borrow to the user. In that case we have runtime crashes + the overhead of RefCell with no benefit... It probably makes sense to use unsafe here to avoid the overhead if we can guarantee that it will never crash.


## Remote design

### How to represent the service in serialized form, and how to deserialize

	For deserialization, use a string corresponding to the type name.

	For the network, we need something that is not necessarily the type name, since one might
	need it to be more unique. Also one might want to have several processes that take the same
	type, but be able to refer uniquely to them.
