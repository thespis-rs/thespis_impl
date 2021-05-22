use crate::{ import::*, BoxEnvelope, StrongCount };

type Rx<A> = Box< dyn Stream<Item=BoxEnvelope<A>> + Send + Unpin >;

/// This wraps a channel receiver in order to do an extra check when the channel returns pending.
/// We want strong and weak addresses. When there are no strong addresses left, we shall return
/// `Poll::Ready(None)` instead of `Poll::Pending`.
///
/// A waker is stored in case the strong count goes to zero while we are already pending.
//
pub struct ChanReceiver<A>
{
	rx   : Rx<A>,
	count: Arc<Mutex< StrongCount >>,
}


impl<A> ChanReceiver<A> where A: Actor
{
	/// Create a new receiver for a mailbox. The `StrongCount` must be a clone from the
	/// one provided to any Addr that are to communicate with this mailbox.
	//
	pub fn new( rx: Rx<A> ) -> Self
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


impl<A> Stream for ChanReceiver<A> where A: Actor
{
	type Item = BoxEnvelope<A>;

	fn poll_next( mut self: Pin<&mut Self>, cx: &mut TaskContext<'_> ) -> Poll< Option<Self::Item> >
	{
		match Pin::new( &mut self.rx ).poll_next( cx )
		{
			Poll::Pending =>
			{
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


impl<A> fmt::Debug for ChanReceiver<A>
{
	fn fmt( &self, fmt: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		fmt.debug_struct( "ChanReceiver<A>" )
		   .field( "count", &self.count )
		   .finish()
	}
}
