use crate::{ import::*, error::*, ChanReceiver };


/// The mailbox implementation.
//
pub struct Mailbox<A> where A: Actor
{
	rx : ChanReceiver<A> ,

	/// This creates a unique id for every mailbox in the program. This way recipients
	/// can impl Eq to say whether they refer to the same actor.
	//
	id   : usize              ,
	name : Option< Arc<str> > ,
}



impl<A> Mailbox<A> where A: Actor
{
	/// Create a new inbox.
	//
	pub fn new( name: Option< Arc<str> >, rx: ChanReceiver<A> ) -> Self
	{
		static MB_COUNTER: AtomicUsize = AtomicUsize::new( 1 );

		// We don't care for ordering here, as long as it's atomic and no two Actor's
		// can have the same id.
		//
		let id = MB_COUNTER.fetch_add( 1, Ordering::Relaxed );

		Self { rx, id, name }
	}


	/// Obtain a `tracing::Span` identifying the actor with it's id and it's name if it has one.
	//
	pub fn span( &self ) -> Span
	{
		if let Some( name ) = &self.name
		{
			trace_span!( "actor", id = self.id, name = name.as_ref() )
		}

		else
		{
			trace_span!( "actor", id = self.id )
		}
	}


	/// Run the mailbox. Returns a future that processes incoming messages. If the
	/// actor panics during message processing, this will return the mailbox to you
	/// so you can supervise actors by respawning your actor and then calling this method
	/// on the mailbox again. All addresses will remain valid in this scenario.
	//
	pub async fn start( mut self, mut actor: A ) -> Option<Self>

		where A: Send
	{
		let span = self.span();

		async
		{
			trace!( "mailbox: started for: {}", &self );

			while let Some( envl ) = self.rx.next().await
			{
				actor.started().await;
				trace!( "actor {} will process a message.", &self );

				if let Err( e ) = AssertUnwindSafe( envl.handle( &mut actor ) ).catch_unwind().await
				{
					error!( "Actor panicked: {}, with error: {:?}", &self, e );
					return Some(self);
				}

				trace!( "actor {} finished handling it's message. Waiting for next message", &self );
			}

			// TODO: this will not be executed when the future for the mailbox get's dropped
			//
			actor.stopped().await;
			trace!( "Mailbox stopped actor for {}", &self );

			None
		}

		.instrument( span )
		.await

	}


	/// Run the mailbox with a non-Send Actor. Returns a future that processes incoming messages. If the
	/// actor panics during message processing, this will return the mailbox to you
	/// so you can supervise actors by respawning your actor and then calling this method
	/// on the mailbox again. All addresses will remain valid in this scenario.
	//
	pub async fn start_local( mut self, mut actor: A ) -> Option<Self>
	{
		let span = self.span();

		async
		{
			actor.started().await;
			trace!( "mailbox: started for: {}", &self );

			while let Some( envl ) = self.rx.next().await
			{
				if let Err( e ) = AssertUnwindSafe( envl.handle_local( &mut actor ) ).catch_unwind().await
				{
					error!( "Actor panicked: {}, with error: {:?}", &self, e );
					return Some(self);
				}

				trace!( "actor {} finished handling it's message. Waiting for next message", &self );
			}

			actor.stopped().await;
			trace!( "Mailbox stopped actor for {}", &self );

			None

		}

		.instrument( span )
		.await
	}


	/// Spawn the mailbox. Use this if you don't want to supervise the actor and don't want a
	/// JoinHandle.
	//
	pub fn spawn( self, actor: A, exec: &impl Spawn ) -> ThesRes<()>

		where A: Send

	{
		let id = self.id;

		Ok( exec.spawn( self.start( actor ).map(|_|()) )

			.map_err( |_e| ThesErr::Spawn{ /* TODO: source: e.into(), */actor: format!("{:?}", id) } )? )
	}


	/// Spawn the mailbox. You get a joinhandle you can await to detect when the mailbox
	/// terminates and which will return you the mailbox if the actor panicked during
	/// message processing. You can use this to supervise the actor.
	///
	/// If you drop the handle, the mailbox will be dropped and the actor will be stopped,
	/// potentially in the middle of processing a message.
	//
	pub fn start_handle( self, actor: A, exec: &impl SpawnHandle< Option<Self> > ) ->

		ThesRes< JoinHandle< Option<Self> > >

		where A: Send

	{
		let id = self.id;

		Ok( exec.spawn_handle( self.start( actor ) )

			.map_err( |_e| ThesErr::Spawn{ /*source: e.into(), */actor: format!("{:?}", id) } )? )
	}


	/// Spawn the mailbox on the current thread. Use this if you don't want to supervise the actor and don't want a
	/// JoinHandle.
	//
	pub fn spawn_local( self, actor: A, exec: &impl LocalSpawn ) -> ThesRes<()>
	{
		let id = self.id;

		Ok( exec.spawn_local( self.start_local( actor ).map(|_|()) )

			.map_err( |_e| ThesErr::Spawn{ /*source: e.into(), */actor: format!("{:?}", id) } )? )
	}


	/// Spawn the mailbox on the current thread. You get a joinhandle you can await to detect when the mailbox
	/// terminates and which will return you the mailbox if the actor panicked during
	/// message processing. You can use this to supervise the actor.
	///
	/// If you drop the handle, the mailbox will be dropped and the actor will be stopped,
	/// potentially in the middle of processing a message.
	//
	pub fn spawn_handle_local( self, actor: A, exec: &impl LocalSpawnHandle< Option<Self> > )

		-> ThesRes< JoinHandle< Option<Self> > >

	{
		let id = self.id;

		Ok( exec.spawn_handle_local( self.start_local( actor ) )

			.map_err( |_e| ThesErr::Spawn{ actor: format!("{:?}", id) } )? )
	}
}



impl<A: Actor> Identify for Mailbox<A>
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


impl<A: Actor> fmt::Debug for Mailbox<A>
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		write!( f, "Mailbox<{}> ~ {}", std::any::type_name::<A>(), &self.id )
	}
}


impl<A: Actor> fmt::Display for Mailbox<A>
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
