use crate :: { import::*, runtime::rt };


// TODO: Ideas for improvement. Create a struct RawAddress, which allows to create other addresses from.
//       Do not hold anything in the mailbox struct. just PhantomData<A>.
//       In method start, create the channel. give msgs to start_fut, and create rawaddress from handle.
//       return rawaddress to user.
//
//       Currently there is no way for the actor to decide to stop itself. Also, currently we give nothing
//       to the actor in the started or stopped method (like a context, a RawAddress).
//
pub struct Inbox<A> where A: Actor
{
	handle: mpsc::UnboundedSender  <BoxEnvelope<A>> ,
	msgs  : mpsc::UnboundedReceiver<BoxEnvelope<A>> ,

	/// This creates a unique id for every mailbox in the program. This way recipients
	/// can impl Eq to say whether they refer to the same actor.
	//
	id    : usize,
}

impl<A> Inbox<A> where A: Actor
{
	pub fn new() -> Self
	{
		static MB_COUNTER: AtomicUsize = AtomicUsize::new( 0 );

		let (handle, msgs) = mpsc::unbounded();

		// TODO: do we need SeqCst or is AcqRel enough?
		//
		let id = MB_COUNTER.fetch_add( 1, Ordering::SeqCst );

		Self { handle, msgs, id }
	}

	pub fn sender( &self ) -> (usize, mpsc::UnboundedSender<BoxEnvelope<A>>)
	{
		(self.id, self.handle.clone())
	}


	pub fn mb_error( e: mpsc::SendError, context: String ) -> ThesErr
	{
		// Can only happen if using bounded channels
		//
		if e.is_full()
		{
			ThesErrKind::MailboxFull{ actor: context }.into()
		}

		// is disconnected
		// Can only happen if the thread in which the mailbox lives panics. Otherwise
		// this addr will keep the mailbox alive.
		//
		else
		{
			ThesErrKind::MailboxClosed{ actor: context }.into()
		}
	}
}




impl<A> Mailbox<A> for Inbox<A> where A: Actor
{
	type Error = ThesErr;

	fn start( self, actor: A ) -> Result<(), Self::Error >
	{
		Ok( rt::spawn_pinned( self.start_fut( actor ) )

			.context( ThesErrKind::Spawn{ context: "Inbox".into() })? )
	}


	fn start_fut( self, mut actor: A ) -> ReturnNoSend<'static, ()>
	{
		Box::pin( async move
		{
			// TODO: Clean this up...
			// We need to drop the handle, otherwise the channel will never close and the program will not
			// terminate. Like this when the user drops all their handles, this loop will automatically break.
			//
			let Inbox{ mut msgs, handle, .. } = self;
			drop( handle );

			await!( actor.started() );
			trace!( "mailbox: started" );

			loop
			{
				match await!( msgs.next() )
				{
					Some( envl ) => { await!( envl.handle( &mut actor ) ); }
					None         => { break;                               }
				}
			}

			await!( actor.stopped() );
			trace!( "Mailbox stopped actor" );

		})
	}
}



impl<A> Default for Inbox<A> where A: Actor
{
	fn default() -> Self { Self::new() }
}
