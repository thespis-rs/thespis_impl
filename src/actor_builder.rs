use crate::{ import::*, BoxEnvelope, ChanSender, ChanReceiver, CloneSinkExt, Addr, ThesErr, Mailbox, MailboxEnd };

/// Default buffer size for bounded channel between Addr and Mailbox.
//
pub const BOUNDED: usize = 16;


/// Builder for [Addr] and [Mailbox]. This is a convenience API so you don't have to call their constructors
/// manually. Mainly lets you set the channel and name for your mailbox.
///
/// Also provides methods for spawning the mailbox immediately as well as a
/// [`build`](ActorBuilder::build) method which lets you do it manually.
///
/// ## Example
///
/// ```rust
/// # futures::executor::block_on(async {
/// use
/// {
///    thespis         :: { *        } ,
///    thespis_impl    :: { *        } ,
///    async_executors :: { AsyncStd } ,
/// };
///
/// #[ derive( Actor ) ]
/// //
/// struct MyActor;
///
/// let (mut addr, mb_handle) = Addr::builder( "my very own actor" )
///    .bounded( Some(24) )
///    .spawn_handle( MyActor, &AsyncStd )?
/// ;
///
/// // mb will end when the last addr is dropped.
/// //
/// drop(addr);
/// mb_handle.await;
///
/// # Ok::<(), ThesErr>(())
/// # }).unwrap();
/// ```
//
pub struct ActorBuilder<A: Actor>
{
	tx     : Option< ChanSender  <A> > ,
	rx     : Option< ChanReceiver<A> > ,
	bounded: Option< usize           > ,
	name   : Arc<str>                  ,
}


impl<A: Actor> ActorBuilder<A>
{
	/// Create a new ActorBuilder with default settings.
	//
	pub fn new( name: impl AsRef<str> ) -> Self
	{
		Self
		{
			tx     : None                 ,
			rx     : None                 ,
			bounded: Some( BOUNDED )      ,
			name   : name.as_ref().into() ,
		}
	}


	/// Choose the bounded size of the default channel. If unset, will default
	/// to a bounded channel with a buffer size of 16. Note that thespis messages
	/// are boxed, so the buffer is only the size of 16 fat pointers. It still means
	/// consuming the actual heap storage if you count on the channel backpressure
	/// to keep your memory consumption down.
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
	/// This option is incompatible with [`ActorBuilder::bounded`].
	///
	/// ## Panics
	/// In debug mode this will panic if you have already called [`ActorBuilder::bounded`].
	///
	/// ## Example
	///
	/// [This example](https://github.com/thespis-rs/thespis_impl/blob/dev/examples/tokio_channel.rs)
	/// shows how to use tokio channels instead.
	//
	pub fn channel<E, TX, RX>( mut self, tx:TX, rx: RX ) -> Self

		where TX: Sink<BoxEnvelope<A>, Error=E> + Clone + Unpin + Send + 'static,
		      E : Error + Sync + Send + 'static,
		      RX: Stream<Item=BoxEnvelope<A>> + Send + Unpin + 'static,
	{
		debug_assert!( self.bounded == Some( BOUNDED ) );

		self.tx = Some( tx.dyned()   );
		self.rx = Some( Box::new(rx) );
		self
	}


	/// Build [`Addr`] and [`Mailbox`]. This does not yet consume an actor and you have to
	/// call [`start`](Mailbox::start) or [`start_local`](Mailbox::start_local) on [`Mailbox`] and spawn the future to
	/// run your actor.
	///
	/// The advantage of this method is that you can pass the Addr to the constructor
	/// of your actor if you need to. Otherwise it's advised to use [`ActorBuilder::spawn_handle`] or [`ActorBuilder::spawn_handle_local`]
	/// for convenience.
	//
	pub fn build( mut self ) -> (Addr<A>, Mailbox<A>)
	{
		if self.rx.is_none() || self.tx.is_none()
		{
			if let Some( bounded ) = self.bounded
			{
				let (tx, rx) = futures::channel::mpsc::channel( bounded );
				self.tx = Some( tx.dyned()   );
				self.rx = Some( Box::new(rx) );
			}

			else
			{
				let (tx, rx) = futures::channel::mpsc::unbounded();
				self.tx = Some( tx.dyned()   );
				self.rx = Some( Box::new(rx) );
			}
		}


		let rx    = self.rx.unwrap();
		let mb    = Mailbox::new( self.name, rx );
		let addr  = mb.addr( self.tx.unwrap() );

		(addr, mb)
	}


	/// Spawn the mailbox on the provided executor. Returns [`Addr`]. Note that this detaches the
	/// mailbox on spawning. That means the actor will keep running until all Addr to it have been
	/// dropped.
	///
	/// If you want more control over stopping the actor, look at [`ActorBuilder::spawn_handle`] or
	/// [`ActorBuilder::build`].
	//
	pub fn spawn( self, actor: A, exec: &dyn Spawn ) -> Result< Addr<A>, ThesErr >

		where A: Send
	{
		let (addr, mb) = self.build();
		let fut = mb.start( actor );

		exec.spawn( async { fut.await; } )

			.map_err( |src| ThesErr::Spawn{ info: addr.info(), src } )?
		;

		Ok(addr)
	}


	/// Spawn the mailbox on the provided executor. Returns [`Addr`] and a [`JoinHandle`] to the spawned
	/// mailbox.
	///
	/// Note that if you drop the [`JoinHandle`] it will stop the actor and drop it unless
	/// you call [`JoinHandle::detach`] on it.
	///
	/// The `JoinHandle` returns:
	/// - [`MailboxEnd::Mailbox`] if you actor panicked whilst processing a message. This way you
	///   can instantiate a new actor and spawn the mailbox again. This can be used for supervision.
	///   All existing addresses will remain valid when this happens.
	/// - [`MailboxEnd::Actor`] when the mailbox ends naturally. You can create a new mailbox and
	///   spawn the actor again if you want to re-use it.
	//
	#[allow(clippy::type_complexity)] // for return type
	//
	pub fn spawn_handle( self, actor: A, exec: & dyn SpawnHandle< MailboxEnd<A> > )

		-> Result< (Addr<A>, JoinHandle< MailboxEnd<A> >), ThesErr >

		where A: Send

	{
		let (addr, mb) = self.build();
		let fut = mb.start( actor );

		let handle = exec.spawn_handle( fut )

			.map_err( |src| ThesErr::Spawn{ info: addr.info(), src } )?
		;

		Ok(( addr, handle ))
	}


	/// For `Actor: !Send`. Spawn the mailbox on the provided executor. Returns the [`Addr`].
	///
	/// If you want more control over stopping the actor, look at [`ActorBuilder::spawn_handle_local`] or
	/// [`ActorBuilder::build`].
	//
	pub fn spawn_local( self, actor: A, exec: & dyn LocalSpawn ) -> Result< Addr<A>, ThesErr >
	{
		let (addr, mb) = self.build();
		let fut = mb.start_local( actor );

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

		let handle = exec.spawn_handle_local( fut )

			.map_err( |src| ThesErr::Spawn{ info: addr.info(), src } )?
		;

		Ok(( addr, handle ))
	}
}




impl<A: Actor> fmt::Debug for ActorBuilder<A>
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		f.debug_struct( "ActorBuilder<A>" )

			.field( "bounded", &self.bounded )
			.field( "name"   , &self.name    )
			.finish()
	}
}
