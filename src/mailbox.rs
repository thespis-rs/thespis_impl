use crate::{ import::*, error::*, Addr, ChanReceiver, RxStrong, ChanSender, ActorInfo };


/// Type returned to you by the mailbox when it ends. Await the JoinHandle returned
/// by the executor to retrieve this.
//
#[ derive(Debug) ]
//
pub enum MailboxEnd<A: Actor>
{
	/// When you get the Actor variant, you actor stopped because all addresses to it were dropped.
	//
	Actor( A ) ,

	/// When you get the Mailbox variant, your actor panicked. You can use this to supervise your actor.
	/// This allows you to instantiate a new actor and spawn it on the same mailbox. The message that
	/// caused the panic will be gone, but other message in the queue will be delivered and all the addresses
	/// to this mailbox remain valid.
	//
	Mailbox( Mailbox<A> ) ,
}



/// The mailbox implementation.
//
pub struct Mailbox<A> where A: Actor
{
	rx  : RxStrong<A>    ,
	info: Arc<ActorInfo> ,
}



impl<A> Mailbox<A> where A: Actor
{
	/// Create a new inbox.
	//
	pub fn new( name: Option<&str>, rx: ChanReceiver<A> ) -> Self
	{
		static MB_COUNTER: AtomicUsize = AtomicUsize::new( 1 );

		// We don't care for ordering here, as long as it's atomic and no two Actor's
		// can have the same id.
		//
		let id = MB_COUNTER.fetch_add( 1, Ordering::Relaxed );

		let rx = RxStrong::new(rx);
		let info = Arc::new( ActorInfo::new::<A>( id, name.map( |n| n.into() ) ) );

		Self { rx, info }
	}


	/// Getter for [`ActorInfo`].
	//
	pub fn info( &self ) -> &ActorInfo
	{
		&self.info
	}



	/// Create an `Addr` to send messages to this mailbox.
	//
	pub fn addr( &self, tx: ChanSender<A> ) -> Addr<A>
	{
		Addr::new( tx, self.info.clone(), self.rx.count() )
	}


	/// Run the mailbox. Returns a future that processes incoming messages. If the
	/// actor panics during message processing, this will return the mailbox to you
	/// so you can supervise actors by re-initiating your actor and then calling this method
	/// on the mailbox again. All addresses will remain valid in this scenario.
	///
	/// This means that we use [`AssertUnwindSafe`](std::panic::AssertUnwindSafe) and
	/// [`catch_unwind`](std::panic::catch_unwind) when calling your actor
	/// and the thread will not unwind. This means that your actor should implement
	/// [`std::panic::UnwindSafe`]. This might become an enforced trait bound for [`Actor`] in
	/// the future.
	///
	/// Warning: if you drop the future returned by this function in order to stop an actor,
	/// [`Actor::stopped`] will not be called.
	//
	pub async fn start( mut self, mut actor: A ) -> MailboxEnd<A>

		where A: Send
	{
		let span = self.info.span();

		async
		{
			actor.started().await;
			trace!( "Mailbox started" );

			while let Some( envl ) = self.rx.next().await
			{
				trace!( "Will process a message." );

				if let Err( e ) = FutureExt::catch_unwind( AssertUnwindSafe( envl.handle( &mut actor ) ) ).await
				{
					error!( "Actor panicked, with error: {:?}", e );
					return MailboxEnd::Mailbox(self);
				}

				trace!( "Finished handling message. Waiting for next message" );
			}

			actor.stopped().await;
			trace!( "Mailbox stopped" );

			MailboxEnd::Actor( actor )
		}

		.instrument( span )
		.await

	}


	/// Run the mailbox with a non-Send Actor. Returns a future that processes incoming messages. If the
	/// actor panics during message processing, this will return the mailbox to you
	/// so you can supervise actors by re-initiating your actor and then calling this method
	/// on the mailbox again. All addresses will remain valid in this scenario.
	///
	/// This means that we use [`AssertUnwindSafe`](std::panic::AssertUnwindSafe) and
	/// [`catch_unwind`](std::panic::catch_unwind) when calling your actor
	/// and the thread will not unwind. This means that your actor should implement
	/// [`std::panic::UnwindSafe`]. This might become an enforced trait bound for [`Actor`] in
	/// the future.
	///
	/// Warning: if you drop the future returned by this function in order to stop an actor,
	/// [`Actor::stopped`] will not be called.
	//
	pub async fn start_local( mut self, mut actor: A ) -> MailboxEnd<A>
	{
		let span = self.info.span();

		async
		{
			actor.started().await;
			trace!( "Mailbox started" );

			while let Some( envl ) = self.rx.next().await
			{
				trace!( "Actor will process a message." );

				if let Err( e ) = FutureExt::catch_unwind( AssertUnwindSafe( envl.handle_local( &mut actor ) ) ).await
				{
					error!( "Actor panicked with error: {:?}", e );
					return MailboxEnd::Mailbox( self );
				}

				trace!( "Actor finished handling it's message. Waiting for next message" );
			}

			actor.stopped().await;
			trace!( "Mailbox stopped" );

			MailboxEnd::Actor( actor )
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
		let info = self.info.clone();

		exec.spawn( self.start(actor).map(|_|()) )

			.map_err( |_e| ThesErr::Spawn( info ) )
	}


	/// Spawn the mailbox. You get a joinhandle you can await to detect when the mailbox
	/// terminates and which will return you the mailbox if the actor panicked during
	/// message processing. You can use this to supervise the actor.
	///
	/// This means that we use [`AssertUnwindSafe`](std::panic::AssertUnwindSafe) and
	/// [`catch_unwind`](std::panic::catch_unwind) when calling your actor
	/// and the thread will not unwind. This means that your actor should implement
	/// [`std::panic::UnwindSafe`]. This might become an enforced trait bound for [`Actor`] in
	/// the future.
	///
	/// If you drop the handle, the mailbox will be dropped and [`Actor::stopped`] will not be called.
	//
	pub fn start_handle( self, actor: A, exec: &impl SpawnHandle< MailboxEnd<A> > ) ->

		ThesRes< JoinHandle< MailboxEnd<A> > >

		where A: Send

	{
		let info = self.info.clone();

		exec.spawn_handle( self.start(actor) )

			.map_err( |_e| ThesErr::Spawn( info ) )
	}


	/// Spawn the mailbox on the current thread. Use this if you don't want to supervise the actor and don't want a
	/// JoinHandle.
	//
	pub fn spawn_local( self, actor: A, exec: &impl LocalSpawn ) -> ThesRes<()>
	{
		let info = self.info.clone();

		exec.spawn_local( self.start_local( actor ).map(|_|()) )

			.map_err( |_e| ThesErr::Spawn( info ) )
	}


	/// Spawn the mailbox on the current thread. You get a joinhandle you can await to detect when the mailbox
	/// terminates and which will return you the mailbox if the actor panicked during
	/// message processing. You can use this to supervise the actor.
	///
	/// This means that we use [`AssertUnwindSafe`](std::panic::AssertUnwindSafe) and
	/// [`catch_unwind`](std::panic::catch_unwind) when calling your actor
	/// and the thread will not unwind. This means that your actor should implement
	/// [`std::panic::UnwindSafe`]. This might become an enforced trait bound for [`Actor`] in
	/// the future.
	///
	/// If you drop the handle, the mailbox will be dropped and [`Actor::stopped`] will not be called.
	//
	pub fn spawn_handle_local( self, actor: A, exec: &impl LocalSpawnHandle< MailboxEnd<A> > )

		-> ThesRes< JoinHandle< MailboxEnd<A> > >

	{
		let info = self.info.clone();

		exec.spawn_handle_local( self.start_local( actor ) )

			.map_err( |_e| ThesErr::Spawn( info ) )
	}
}



impl<A: Actor> Identify for Mailbox<A>
{
	fn id( &self ) -> usize
	{
		self.info.id
	}



	fn name( &self ) -> Option<Arc<str>>
	{
		self.info.name.clone()
	}
}


impl<A: Actor> fmt::Debug for Mailbox<A>
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		write!( f, "Mailbox<{}> ~ {}", std::any::type_name::<A>(), &self.info.id )
	}
}


impl<A: Actor> fmt::Display for Mailbox<A>
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		match &self.info.name
		{
			Some(n) => write!( f, "{} ({}, {})", self.info.type_name(), self.info.id, n ) ,
			None    => write!( f, "{} ({})"    , self.info.type_name(), self.info.id    ) ,
		}
	}
}
