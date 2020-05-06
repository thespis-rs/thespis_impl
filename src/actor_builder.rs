use crate::{ import::*, ChanSender, ChanReceiver, Addr, ThesErr, Inbox, SinkError };

/// Default buffer size for bounded channel between Addr and Inbox.
//
pub const BOUNDED: usize = 16;

/// Builder for Addr and Inbox.
//
pub struct ActorBuilder<A: Actor>
{
	tx     : Option< ChanSender  <A> > ,
	rx     : Option< ChanReceiver<A> > ,
	bounded: Option< usize           > ,
	name   : Option< Arc<str>        > ,
}



impl<A: Actor> Default for ActorBuilder<A>
{
	fn default() -> Self
	{
		Self
		{
			tx     : None ,
			rx     : None ,
			bounded: Some( BOUNDED      ) ,
			name   : Some( "N/A".into() ) ,
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
	pub fn name( mut self, name: Arc<str> ) -> Self
	{
		self.name = name.into();
		self
	}


	/// Choose the bounded size of the default channel. If unset, will default
	/// to a bounded channel with a buffer size of
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

		self.bounded = bounded.into();
		self
	}


	/// Set the channel to use for communication between `Addr` and `Inbox`.
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


	/// Build [`Addr`] and [`Inbox`]. This does not yet consume an actor and you have to
	/// call [`start`] or [`start_local`] on [`Inbox`] and spawn the future to
	/// run your actor.
	///
	/// The advantage of this method is that you can pass the Addr to the constructor
	/// of your actor if you need to. Otherwise it's advised to use [`spawn`] or [`spawn_local`]
	/// for convenience.
	//
	pub fn build( mut self ) -> (Addr<A>, Inbox<A>)
	{
		#[ cfg( feature = "tokio_channel" ) ]
		//
		if self.rx.is_none() || self.tx.is_none()
		{
			use async_chanx::{ TokioSender, TokioUnboundedSender };

			if let Some( bounded ) = self.bounded
			{
				let (tx, rx) = tokio::sync::mpsc::channel( bounded );
				let tx = Box::new( TokioSender::new( tx ).sink_map_err( |e| -> SinkError { Box::new(e) } ) );

				self.tx = Some( Box::new(tx) );
				self.rx = Some( Box::new(rx) );
			}

			else
			{
				let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
				let tx = Box::new( TokioUnboundedSender::new( tx ).sink_map_err( |e| -> SinkError { Box::new(e) } ) );

				self.tx = Some( Box::new(tx) );
				self.rx = Some( Box::new(rx) );
			}
		}


		#[ cfg(not( feature = "tokio_channel" )) ]
		//
		if self.rx.is_none() || self.tx.is_none()
		{
			if let Some( bounded ) = self.bounded
			{
				let (tx, rx) = futures::channel::mpsc::channel( bounded );
				let tx       = Box::new( tx.sink_map_err( |e| -> SinkError { Box::new(e) } ) );

				self.tx = Some( Box::new(tx) );
				self.rx = Some( Box::new(rx) );
			}

			else
			{
				let (tx, rx) = futures::channel::mpsc::unbounded();
				let tx       = Box::new( tx.sink_map_err( |e| -> SinkError { Box::new(e) } ) );

				self.tx = Some( Box::new(tx) );
				self.rx = Some( Box::new(rx) );
			}
		}

		let mb   = crate::Inbox::new( self.name, self.rx.unwrap() );
		let addr = Addr::new( mb.id(), mb.name(), self.tx.unwrap() );

		(addr, mb)
	}


	/// Spawn the mailbox on the provided executor. Returns [`Addr`]. Note that this detaches the
	/// mailbox on spawning. That means the actor will keep running until all Addr to it have been
	/// dropped.
	///
	/// If you want more control over stopping the actor, look at [`ActorBuilder::start_handle`] or
	/// [`ActorBuilder::build`].
	//
	pub fn start( self, actor: A, exec: &dyn Spawn ) -> Result< Addr<A>, ThesErr >

		where A: Send
	{
		let (addr, mb) = self.build();
		let fut = mb.start( actor );

		// Todo, include a source error.
		//
		exec.spawn( async { fut.await; } ).map_err( |_| ThesErr::Spawn{ actor: format!( "inbox id: {:?}", addr.id() ) } )?;

		Ok(addr)
	}


	/// Spawn the mailbox on the provided executor. Returns [`Addr`] and a [`JoinHandle`] to the spawned
	/// mailbox. Note that if you drop the [`JoinHandle`] it will stop the actor and drop it unless
	/// you call [`JoinHandle::detach`] on it.
	//
	pub fn start_handle( self, actor: A, exec: & dyn SpawnHandle< Option<Inbox<A>> > )

		-> Result< (Addr<A>, JoinHandle< Option<Inbox<A>> >), ThesErr >

		where A: Send

	{
		let (addr, mb) = self.build();
		let fut = mb.start( actor );

		// Todo, include a source error.
		//
		let handle = exec.spawn_handle( fut ).map_err( |_| ThesErr::Spawn{ actor: format!( "inbox id: {:?}", addr.id() ) } )?;

		Ok(( addr, handle ))
	}


	/// For `Actor: !Send`. Spawn the mailbox on the provided executor. Returns the [`Addr`].
	///
	/// If you want more control over stopping the actor, look at [`ActorBuilder::start_handle_local`] or
	/// [`ActorBuilder::build`].
	//
	pub fn start_local( self, actor: A, exec: & dyn LocalSpawn ) -> Result< Addr<A>, ThesErr >
	{
		let (addr, mb) = self.build();
		let fut = mb.start_local( actor );

		// Todo, include a source error.
		//
		exec.spawn_local( async { fut.await; } ).map_err( |_| ThesErr::Spawn{ actor: format!( "inbox id: {:?}", addr.id() ) } )?;

		Ok(addr)
	}


	/// For `Actor: !Send`. Spawn the mailbox on the provided executor. Returns [`Addr`] and a [`JoinHandle`] to the spawned
	/// mailbox. Note that if you drop the [`JoinHandle`] it will stop the actor and drop it unless
	/// you call [`JoinHandle::detach`] on it.
	//
	pub fn start_handle_local( self, actor: A, exec: & dyn LocalSpawnHandle< Option<Inbox<A>> > )

		-> Result< (Addr<A>, JoinHandle< Option<Inbox<A>> >), ThesErr >

	{
		let (addr, mb) = self.build();
		let fut = mb.start_local( actor );

		// Todo, include a source error.
		//
		let handle = exec.spawn_handle_local( fut ).map_err( |_| ThesErr::Spawn{ actor: format!( "inbox id: {:?}", addr.id() ) } )?;

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
