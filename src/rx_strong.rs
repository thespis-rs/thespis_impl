use crate::{ import::*, BoxEnvelope, ChanReceiver, StrongCount };


/// This wraps a channel receiver in order to do an extra check when the channel returns pending.
/// We want strong and weak addresses. When there are no strong addresses left, we shall return
/// `Poll::Ready(None)` instead of `Poll::Pending`.
///
/// A waker is stored in case the strong count goes to zero while we are already pending.
//
pub(crate) struct RxStrong<A>
{
	rx   : ChanReceiver<A>,
	count: Arc<Mutex< StrongCount >>,
}


impl<A> RxStrong<A> where A: Actor
{
	/// Create a new receiver for a mailbox. The `StrongCount` must be a clone from the
	/// one provided to any Addr that are to communicate with this mailbox.
	//
	pub(crate) fn new( rx: ChanReceiver<A> ) -> Self
	{
		let count = Arc::new( Mutex::new( StrongCount::new() ) );
		Self{ rx, count }
	}


	/// Access the strong count.
	//
	pub(crate) fn count( &self ) -> Arc<Mutex< StrongCount >>
	{
		self.count.clone()
	}
}


impl<A> Stream for RxStrong<A> where A: Actor
{
	type Item = BoxEnvelope<A>;

	fn poll_next( mut self: Pin<&mut Self>, cx: &mut TaskContext<'_> ) -> Poll< Option<Self::Item> >
	{
		let size_hint = self.rx.size_hint();

		match Pin::new( &mut self.rx ).poll_next( cx )
		{
			Poll::Pending =>
			{
				trace!( "size hint is: {:?}", size_hint );

				let count = self.count.lock().expect( "Mutex<StrongCount> poisoned" );

				if count.count() == 0
				{
					Poll::Ready( None )
				}

				else
				{
					// Tell the StrongCount to wake us up in case the count goes to zero.
					//
					count.store_waker( cx.waker() );

					Poll::Pending
				}
			}

			// pass through anything but Pending to the channel.
			//
			x => x,
		}
	}

	fn size_hint( &self ) -> (usize, Option<usize>)
	{
		self.rx.size_hint()
	}
}


impl<A> fmt::Debug for RxStrong<A>
{
	fn fmt( &self, fmt: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		fmt.debug_struct( "RxStrong<A>" )
		   .field( "count", &self.count )
		   .finish()
	}
}
