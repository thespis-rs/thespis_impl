//! Thespis_impl supports both strong and weak addresses. When all strong addresses to a mailbox are
//! dropped, the mailbox will shut down.
//!
//! The logic is that sending is already fallible, so a weak address will deny sending with an error
//! messages if the strong count is zero. However the mailbox will still continue processing all messages
//! already in the queue and stop as soon as the queue is empty.
//!
//! This solution must support the following scenarios:
//! - WeakAddr must be able to check the count before allowing sending.
//! - When a WeakAddr is pending on a full queue and strong count goes to zero, should we return error?
//!   If we err, can we let the user recover their message? What is then the type of the message in the
//!   error enum? It would mean that the weak address has to be woken up in from the pending state when
//!   strong count goes to zero. -> Seems like pretty complicated, better consider that if the weak address already
//!   accepted the message, it will now be processed.
//! - A WeakAddr must be able to upgrade to strong as long as the strong count has never been zero.
//! - When the mailbox is pending, eg. queue is empty and strong count goes to zero, it must be woken up.
//! - When the mailbox is processing and strong count goes to zero, when it becomes pending, eg. the
//!   queue is now empty it must be able to check that and must be able to break from the pending state and shut down.
//! - Strong addresses must be able to decrement the counter in Drop, eg. non-async context.
//!
//! Current solution has us wrapping the channel receiver to intercept the `Pending` state. It will then
//! check the strong count, and if it should remain alive, register it's waker so the task gets woken
//! up in case the count goes to zero.
//!
use core::task::Waker;
use crate::{ import::* };
use futures::task::AtomicWaker;
use std::sync::atomic::{ AtomicUsize, Ordering };

#[ derive( Debug, Default ) ]
//
struct Inner
{
	count: AtomicUsize,
	waker: AtomicWaker,
}

/// Type that represents an async counter that will keep track of how many `Addr` exist for a mailbox.
/// This allows to shut down the mailbox when there are only `WeakAddr` left alive.
///
/// This can be cloned to give copies to both `Addr::new` and `ChanReceiver::new`.
//
#[ derive( Clone, Debug, Default ) ]
//
pub(crate) struct StrongCount
{
	inner: Arc<Inner>
}


impl StrongCount
{
	/// Create a new `StrongCount`.
	//
	pub(crate) fn new() -> Self
	{
		Default::default()
	}

	/// Read the current count.
	//
	pub(crate) fn count( &self ) -> usize
	{
		self.inner.count.load( Ordering::Relaxed )
	}

	/// Increment the count of strong addresses. Returns the old value.
	//
	pub(crate) fn increment( &self ) -> usize
	{
		self.inner.count.fetch_add( 1, Ordering::Relaxed )
	}

	/// Decrement the count of strong addresses.
	//
	pub(crate) fn decrement( &self )
	{
		let old = self.inner.count.fetch_sub( 1, Ordering::Relaxed );

		// if the value ever is smaller than zero we have a bug. You cannot
		// drop more addresses than were ever created.
		//
		debug_assert!( old >= 1 );

		// Now we are at zero.
		//
		if old == 1
		{
			self.inner.waker.wake();
		}
	}

	/// Register a waker to be woken up when the count reaches zero.
	//
	pub(crate) fn store_waker( &self, waker: &Waker )
	{
		self.inner.waker.register( waker );
	}
}
