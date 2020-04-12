use crate::{ import::*, ChanSender, BoxEnvelope, envelope::*, error::* };



/// Reference implementation of thespis::Address<A, M>.
/// It can receive all message types the actor implements thespis::Handler for.
/// An actor will be dropped when all addresses to it are dropped.
//
pub struct Addr< A: Actor >
{
	mb  : ChanSender<A>      ,
	id  : usize              ,
	name: Option< Arc<str> > ,
}



impl< A: Actor > Clone for Addr<A>
{
	fn clone( &self ) -> Self
	{
		trace!( "CREATE address for: {}", self );

		Self { mb: self.mb.clone_sink(), id: self.id, name: self.name.clone() }
	}
}


/// Verify whether 2 Receivers will deliver to the same actor.
//
impl< A: Actor > PartialEq for Addr<A>
{
	fn eq( &self, other: &Self ) -> bool
	{
		self.id == other.id
	}
}

impl< A: Actor > Eq for Addr<A>{}



impl<A: Actor> fmt::Debug for Addr<A>
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		let name = match &self.name
		{
			Some( s ) => format!( ", {}", s ) ,
			None      => String::new()        ,
		};

		write!
		(
			f                          ,
			"Addr<{}> ~ {}{}"          ,
			std::any::type_name::<A>() ,
			&self.id                   ,
			name                       ,
		)
	}
}



impl<A: Actor> fmt::Display for Addr<A>
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




impl<A> Addr<A> where A: Actor
{
	/// Create a new address. The simplest way is to use Addr::try_from( Actor ).
	/// This way allows more control. You need to manually make the mailbox. See the
	/// no_rt example in the repository.
	//
	pub fn new( id: usize, name: Option< Arc<str> >, tx: ChanSender<A> ) -> Self
	{
		let new = Self{ id, name, mb: tx };

		trace!( "CREATE address for: {}", &new );

		new
	}


	/// Automatically create a mailbox (thespis_impl::single_thread::Inbox) and an address from your
	/// actor. This avoids the boilerplate of manually having to create the mailbox and the address.
	/// Will consume your actor and return an address.
	///
	/// It uses a [`tokio::sync::mpsc`] channel with a 16 msg buffer.
	///
	/// ## Errors
	///
	/// - [`ThesErr::Spawn`] with the id of the mailbox when spawning fails.
	///
	///
	/// TODO: have doc examples tested by rustdoc
	///
	/// ```ignore
	/// let addr = Addr::try_from( MyActor{} )?;
	/// addr.call( MyMessage{} ).await?;
	/// ```
	//
	#[ cfg( feature = "convenience" ) ]
	//
	pub fn try_from_actor( actor: A, exec: impl Spawn ) -> ThesRes<Self> where A: Send
	{
		use async_chanx::TokioSender;
		use crate::{ SinkError, Inbox };

		let (tx, rx) = tokio::sync::mpsc::channel( 16 );
		let mb       = Inbox::new( None, Box::new(rx) ) ;
		let id       = mb.id();

		let tx    = Box::new( TokioSender::new( tx ).sink_map_err( |e| -> SinkError { Box::new(e) } ) );
		let addr  = Addr::new( id, None, tx ) ;

		exec.spawn( mb.start_fut( actor ) ).map_err( |_| ThesErr::Spawn{ actor: format!( "{}", id ) } )?;
		Ok( addr )
	}

	/// Automatically create a mailbox (thespis_impl::single_thread::Inbox) and an address from your
	/// actor. This avoids the boilerplate of manually having to create the mailbox and the address.
	/// Will consume your actor and return an address.
	///
	/// This will spawn the mailbox on the current thread. You need to set up async_runtime to enable
	/// spawn_local.
	///
	/// It uses a [`tokio::sync::mpsc`] channel with a 16 msg buffer.
	///
	/// ## Errors
	///
	/// - [`ThesErr::Spawn`] with the id of the mailbox when spawning fails.
	///
	/// TODO: have doc examples tested by rustdoc
	///
	/// ```ignore
	/// let addr = Addr::try_from( MyActor{} )?;
	/// addr.call( MyMessage{} ).await?;
	/// ```
	//
	#[ cfg( feature = "convenience" ) ]
	//
	pub fn try_from_actor_local( actor: A, exec: &impl LocalSpawn ) -> ThesRes<Self>
	{
		use async_chanx::TokioSender;
		use crate::{ SinkError, Inbox };

		let (tx, rx) = tokio::sync::mpsc::channel( 16 );
		let mb       = Inbox::new( None, Box::new(rx) ) ;
		let id       = mb.id();

		let tx    = Box::new( TokioSender::new( tx ).sink_map_err( |e| -> SinkError { Box::new(e) } ) );
		let addr  = Addr::new( id, None, tx ) ;

		exec.spawn_local( mb.start_fut_local( actor ) ).map_err( |_| ThesErr::Spawn{ actor: format!( "{}", id ) } )?;
		Ok( addr )
	}
}

// For debugging
//
impl<A: Actor> Drop for Addr<A>
{
	fn drop( &mut self )
	{
		trace!( "DROP address for: {}", self );
	}
}



impl<A, M> Address<M> for Addr<A>

	where  A: Actor + Handler<M> ,
	       M: Message            ,

{
	fn call( &mut self, msg: M ) -> Return<'_, ThesRes< <M as Message>::Return >>
	{
		async move
		{
			let (ret_tx, ret_rx)     = oneshot::channel::<M::Return>()              ;
			let envl: BoxEnvelope<A> = Box::new( CallEnvelope::new( msg, ret_tx ) ) ;
			let result               = self.mb.send( envl ).await                   ;

			// MailboxClosed
			//
			result.map_err( |_| ThesErr::MailboxClosed{ actor: format!("{:?}", self) } )?;


			ret_rx.await

				.map_err( |_| ThesErr::MailboxClosedBeforeResponse{ actor: format!( "{:?}", self ) }.into() )

		}.boxed()
	}



	fn clone_box<'a>( &self ) -> BoxAddress<'a, M, ThesErr>
	{
		Box::new( self.clone() )
	}
}


impl<A> Identify for Addr<A>

	where  A: Actor,

{
	/// Get the id of the mailbox this address sends to. There will be exactly one for each
	/// actor, so you can use this for uniquely identifying your actors.
	///
	/// This is an atomic usize that is incremented for every new mailbox. There currently
	/// is no overflow protection.
	//
	fn id( &self ) -> usize
	{
		self.id
	}

	fn name( &self ) -> Option< Arc<str> >
	{
		self.name.clone()
	}
}



impl<A, M> Sink<M> for Addr<A>

	where A: Actor + Handler<M> ,
	      M: Message            ,

{
	type Error = ThesErr;

	fn poll_ready( mut self: Pin<&mut Self>, cx: &mut TaskContext<'_> ) -> Poll<Result<(), Self::Error>>
	{
		match Pin::new( &mut self.mb ).poll_ready( cx )
		{
			Poll::Ready( p ) => match p
			{
				Ok (_) => Poll::Ready( Ok(()) ),
				Err(_) =>
				{
					Poll::Ready( Err( ThesErr::MailboxClosed{ actor: format!("{:?}", self) } ) )
				}
			}

			Poll::Pending => Poll::Pending
		}
	}


	fn start_send( mut self: Pin<&mut Self>, msg: M ) -> Result<(), Self::Error>
	{
		let envl: BoxEnvelope<A>= Box::new( SendEnvelope::new( msg ) );

		Pin::new( &mut self.mb )

			.start_send( envl )

			// if poll_ready wasn't called, the underlying code panics in tokio-sync.
			//
			.map_err( |_| ThesErr::MailboxClosed{ actor: format!("{:?}", self) } )
	}


	fn poll_flush( mut self: Pin<&mut Self>, cx: &mut TaskContext<'_> ) -> Poll<Result<(), Self::Error>>
	{
		match Pin::new( &mut self.mb ).poll_flush( cx )
		{
			Poll::Ready( p ) => match p
			{
				Ok (_) => Poll::Ready( Ok(()) ),
				Err(_) =>
				{
					Poll::Ready( Err( ThesErr::MailboxClosed{ actor: format!("{:?}", self) } ))
				}
			}

			Poll::Pending => Poll::Pending
		}
	}


	/// This is a no-op. The address can only really close when dropped. Close has no meaning before that.
	//
	fn poll_close( self: Pin<&mut Self>, _cx: &mut TaskContext<'_> ) -> Poll<Result<(), Self::Error>>
	{
		Ok(()).into()
	}
}
