use crate::{ import::*, ChanSender, ChanReceiver, Addr, ThesErr, Mailbox, MailboxEnd, DynError };

/// Default buffer size for bounded channel between Addr and Mailbox.
//
pub const BOUNDED: usize = 16;


/// Builder for Addr and Mailbox. This is a convenience API so you don't have to call their constructors
/// manually. Mainly lets you set the channel and name for your mailbox.
///
/// This provides the same methods as [`Mailbox`] for spawning the mailbox immediately as well as a
/// [`build`](ActorBuilder::build) method which let's you manually spawn the mailbox.
//
pub struct ActorBuilder<A: Actor>
{
	tx     : Option< ChanSender  <A> > ,
	rx     : Option< ChanReceiver<A> > ,
	bounded: Option< usize           > ,
	name   : Option< String          > ,
}



impl<A: Actor> Default for ActorBuilder<A>
{
	fn default() -> Self
	{
		Self
		{
			tx     : None            ,
			rx     : None            ,
			bounded: Some( BOUNDED ) ,
			name   : None            ,
		}
	}
}



impl<A: Actor> ActorBuilder<A>
{
	/// Create a new ActorBuilder with default settings.
	//
	pub fn new() -> Self
	{
		Self::default()
	}


	/// Configure a name for this actor. This will be helpful for interpreting
	/// debug logs. You can also retrieve the name later on the Addr.
	//
	pub fn name( mut self, name: &str ) -> Self
	{
		self.name = Some( name.into() );
		self
	}


	/// Choose the bounded size of the default channel. If unset, will default
	/// to a bounded channel with a buffer size of 16.
	///
	/// If you set this to `None`, you will get an unbounded channel.
	///
	/// This option is incompatible with manually providing a channel. Only
	/// works for the default channel.
	///
	/// ## Panics
	/// In debug mode this will panic if you have already called [`ActorBuilder::channel`].
	//
	pub fn bounded( mut self, bounded: Option<usize> ) -> Self
	{
		debug_assert!( self.tx.is_none() );
		debug_assert!( self.rx.is_none() );

		self.bounded = bounded;
		self
	}


	/// Set the channel to use for communication between `Addr` and `Mailbox`.
	///
	/// This option is incompatible with bounded.
	///
	/// ## Panics
	/// In debug mode this will panic if you have already called [`ActorBuilder::bounded`].
	//
	pub fn channel( mut self, tx: ChanSender<A>, rx: ChanReceiver<A> ) -> Self
	{
		debug_assert!( self.bounded == Some( BOUNDED ) );

		self.tx = tx.into();
		self.rx = rx.into();
		self
	}


	/// Build [`Addr`] and [`Mailbox`]. This does not yet consume an actor and you have to
	/// call [`start`](Mailbox::start) or [`start_local`](Mailbox::start_local) on [`Mailbox`] and spawn the future to
	/// run your actor.
	///
	/// The advantage of this method is that you can pass the Addr to the constructor
	/// of your actor if you need to. Otherwise it's advised to use [`ActorBuilder::start_handle`] or [`ActorBuilder::start_handle_local`]
	/// for convenience.
	//
	pub fn build( mut self ) -> (Addr<A>, Mailbox<A>)
	{
		if self.rx.is_none() || self.tx.is_none()
		{
			if let Some( bounded ) = self.bounded
			{
				let (tx, rx) = futures::channel::mpsc::channel( bounded );
				let tx       = Box::new( tx.sink_map_err( |e| -> DynError { Box::new(e) } ) );

				self.tx = Some( Box::new(tx) );
				self.rx = Some( Box::new(rx) );
			}

			else
			{
				let (tx, rx) = futures::channel::mpsc::unbounded();
				let tx       = Box::new( tx.sink_map_err( |e| -> DynError { Box::new(e) } ) );

				self.tx = Some( Box::new(tx) );
				self.rx = Some( Box::new(rx) );
			}
		}


		let rx    = self.rx.unwrap();
		let mb    = Mailbox::new( self.name.as_deref(), rx );
		let addr  = mb.addr( self.tx.unwrap() );

		(addr, mb)
	}


	/// Spawn the mailbox on the provided executor. Returns [`Addr`]. Note that this detaches the
	/// mailbox on spawning. That means the actor will keep running until all Addr to it have been
	/// dropped.
	///
	/// If you want more control over stopping the actor, look at [`ActorBuilder::start_handle`] or
	/// [`ActorBuilder::build`].
	//
	pub fn spawn( self, actor: A, exec: &dyn Spawn ) -> Result< Addr<A>, ThesErr >

		where A: Send
	{
		let (addr, mb) = self.build();
		let fut = mb.start( actor );

		// Todo, include a source error.
		//
		exec.spawn( async { fut.await; } )

			.map_err( |src| ThesErr::Spawn{ info: addr.info(), src } )?
		;

		Ok(addr)
	}


	/// Spawn the mailbox on the provided executor. Returns [`Addr`] and a [`JoinHandle`] to the spawned
	/// mailbox.
	///
	/// Note that if you drop the [`JoinHandle`] it will stop the actor and drop it unless
	/// you call [`JoinHandle::detach`] on it. If the actor panics during message processing, the JoinHandle
	/// will return to you the mailbox so you can instantiate a new actor for the mailbox. The address
	/// will remain valid and you can use this property to supervise the actor.
	//
	#[allow(clippy::type_complexity)] // for return type
	//
	pub fn spawn_handle( self, actor: A, exec: & dyn SpawnHandle< MailboxEnd<A> > )

		-> Result< (Addr<A>, JoinHandle< MailboxEnd<A> >), ThesErr >

		where A: Send

	{
		let (addr, mb) = self.build();
		let fut = mb.start( actor );

		// Todo, include a source error.
		//
		let handle = exec.spawn_handle( fut )

			.map_err( |src| ThesErr::Spawn{ info: addr.info(), src } )?
		;

		Ok(( addr, handle ))
	}


	/// For `Actor: !Send`. Spawn the mailbox on the provided executor. Returns the [`Addr`].
	///
	/// If you want more control over stopping the actor, look at [`ActorBuilder::start_handle_local`] or
	/// [`ActorBuilder::build`].
	//
	pub fn spawn_local( self, actor: A, exec: & dyn LocalSpawn ) -> Result< Addr<A>, ThesErr >
	{
		let (addr, mb) = self.build();
		let fut = mb.start_local( actor );

		// Todo, include a source error.
		//
		exec.spawn_local( async { fut.await; } )

			.map_err( |src| ThesErr::Spawn{ info: addr.info(), src } )?
		;

		Ok(addr)
	}


	/// For `Actor: !Send`. Spawn the mailbox on the provided executor. Returns [`Addr`] and a [`JoinHandle`] to the spawned
	/// mailbox.
	///
	/// Note that if you drop the [`JoinHandle`] it will stop the actor and drop it unless
	/// you call [`JoinHandle::detach`] on it.
	//
	#[allow(clippy::type_complexity)] // for return type
	//
	pub fn spawn_handle_local( self, actor: A, exec: & dyn LocalSpawnHandle< MailboxEnd<A> > )

		-> Result< (Addr<A>, JoinHandle< MailboxEnd<A> >), ThesErr >

	{
		let (addr, mb) = self.build();
		let fut = mb.start_local( actor );

		// Todo, include a source error.
		//
		let handle = exec.spawn_handle_local( fut )

			.map_err( |src| ThesErr::Spawn{ info: addr.info(), src } )?
		;

		Ok(( addr, handle ))
	}
}


// TODO: Print out fields.
//
impl<A: Actor> fmt::Debug for ActorBuilder<A>
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		write!( f, "ActorBuilder" )
	}
}
