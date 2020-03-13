use crate::{ import::*, error::* };


/// The mailbox implementation.
//
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
	id    : usize              ,
	name  : Option< Arc<str> > ,
}



impl<A> Inbox<A> where A: Actor
{
	/// Create a new inbox.
	//
	pub fn new( name: Option< Arc<str> > ) -> Self
	{
		static MB_COUNTER: AtomicUsize = AtomicUsize::new( 1 );

		let (handle, msgs) = mpsc::unbounded();

		// We don't care for ordering here, as long as it's atomic and no two Actor's
		// can have the same id.
		//
		let id = MB_COUNTER.fetch_add( 1, Ordering::Relaxed );

		Self
		{
			handle ,
			msgs   ,
			id     ,
			name   ,
		}
	}

	/// A handle to a channel sender that can be used for creating an address to this mailbox.
	//
	pub fn sender( &self ) -> (usize, Option< Arc<str> >, mpsc::UnboundedSender<BoxEnvelope<A>>)
	{
		(self.id, self.name.clone(), self.handle.clone())
	}


	/// Translate a futures SendError to a ThesErr.
	/// Returns MailboxFull or MailboxClosed.
	//
	pub fn mb_error( e: mpsc::SendError, context: String ) -> ThesErr
	{
		// Can only happen if using bounded channels
		//
		if e.is_full()
		{
			ThesErr::MailboxFull{ actor: context }.into()
		}

		// is disconnected
		// Can only happen if the thread in which the mailbox lives panics. Otherwise
		// this addr will keep the mailbox alive.
		//
		else
		{
			ThesErr::MailboxClosed{ actor: context }.into()
		}
	}



	async fn start_fut_inner( mut self, mut actor: A ) where A: Send
	{
		// We need to disconnect the handle, otherwise the channel will never close and the program will not
		// terminate. Like this when the user drops all their handles, this loop will automatically break.
		//
		self.handle.disconnect();

		actor.started().await;
		trace!( "mailbox: started for: {}", &self );

		while let Some( envl ) = self.msgs.next().await
		{
			trace!( "actor {} will process a message.", &self );

			envl.handle( &mut actor ).await;

			trace!( "actor {} finished handling it's message. Waiting for next message", &self );
		}

		// TODO: this will not be executed when the future for the mailbox get's dropped
		//
		actor.stopped().await;
		trace!( "Mailbox stopped actor for {}", &self );
	}



	async fn start_fut_inner_local( mut self, mut actor: A )
	{
		// We need to disconnect the handle, otherwise the channel will never close and the program will not
		// terminate. Like this when the user drops all their handles, this loop will automatically break.
		//
		self.handle.disconnect();

		actor.started().await;
		trace!( "mailbox: started for: {}", &self );

		while let Some( envl ) = self.msgs.next().await
		{
			envl.handle_local( &mut actor ).await;
			trace!( "actor {} finished handling it's message. Waiting for next message", &self );
		}

		actor.stopped().await;
		trace!( "Mailbox stopped actor for {}", &self );
	}


	/// Spawn the mailbox.
	/// TODO: remove the start and start_local
	//
	pub fn start( self, actor: A, exec: &impl Spawn ) -> ThesRes<()> where A: Send
	{
		let id = self.id;

		Ok( exec.spawn( self.start_fut( actor ) )

			.map_err( |_e| ThesErr::Spawn{ /*source: e.into(), */actor: format!("{:?}", id) } )? )
	}


	/// Spawn the mailbox on the current thread.
	//
	pub fn start_local( self, actor: A, exec: &impl LocalSpawn ) -> ThesRes<()>
	{
		let id = self.id;

		Ok( exec.spawn_local( self.start_fut_inner_local( actor ) )

			.map_err( |_e| ThesErr::Spawn{ /*source: e.into(), */actor: format!("{:?}", id) } )? )
	}
}



impl<A: Actor + Send> Mailbox<A> for Inbox<A>
{
	fn start_fut( self, actor: A ) -> Return<'static, ()>
	{
		self.start_fut_inner( actor ).boxed()
	}
}



impl<A: Actor> MailboxLocal<A> for Inbox<A>
{
	fn start_fut_local( self, actor: A ) -> ReturnNoSend<'static, ()>
	{
		self.start_fut_inner_local( actor ).boxed_local()
	}
}



impl<A: Actor> Identify for Inbox<A>
{
	fn id( &self ) -> usize
	{
		self.id
	}



	fn name( &self ) -> Option<Arc<str>>
	{
		self.name.clone()
	}
}



impl<A> Default for Inbox<A> where A: Actor
{
	fn default() -> Self { Self::new( None ) }
}



impl<A: Actor> fmt::Debug for Inbox<A>
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		write!( f, "Inbox<{}> ~ {}", std::any::type_name::<A>(), &self.id )
	}
}


impl<A: Actor> fmt::Display for Inbox<A>
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		// TODO: check expect.
		//
		let t = std::any::type_name::<A>().split( "::" ).last().expect( "there to be no types on the root namespace" );

		match &self.name
		{
			Some(n) => write!( f, "{} ({}, {})", t, self.id, n ) ,
			None    => write!( f, "{} ({})"    , t, self.id    ) ,
		}
	}
}
