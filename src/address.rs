use crate::{ import::*, Inbox, envelope::*, error::* };



/// Reference implementation of thespis::Address<A, M>.
/// It can receive all message types the actor implements thespis::Handler for.
/// An actor will be dropped when all addresses to it are dropped.
//
pub struct Addr< A: Actor >
{
	mb: mpsc::UnboundedSender< BoxEnvelope<A> >,
	id: usize                                  ,
}



impl< A: Actor > Clone for Addr<A>
{
	fn clone( &self ) -> Self
	{
		trace!( "CREATE address for: {} ~ {}", clean_name( std::any::type_name::<A>() ), self.id );

		Self { mb: self.mb.clone(), id: self.id }
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
		write!( f, "Addr<{}> ~ {}", clean_name( std::any::type_name::<A>() ), &self.id )
	}
}


/// Remove all paths from names
//
fn clean_name( name: &str ) -> String
{
	use regex::Regex;

	let re = Regex::new( r"\w+::" ).unwrap();
	let s  = re.replace_all( name, "" );

	// this is just a specific one when using the Peer from remote
	//
	s.replace
	(
		"Peer<Compat01As03Sink<SplitSink<Framed<TcpStream, MulServTokioCodec<MultiServiceImpl<ServiceID, ConnID, Codecs>>>>, MultiServiceImpl<ServiceID, ConnID, Codecs>>, MultiServiceImpl<ServiceID, ConnID, Codecs>>",
		"Peer"
	)
}



impl<A> Addr<A> where A: Actor
{
	/// Create a new address. The simplest way is to use Addr::try_from( Actor ).
	/// This way allows more control. You need to manually make the mailbox. See the
	/// no_rt example in the repository.
	///
	// TODO: take a impl trait instead of a concrete type. This leaks impl details.
	//
	pub fn new( mb: (usize, mpsc::UnboundedSender<BoxEnvelope<A>>) ) -> Self
	{
		trace!( "CREATE address for: {}", clean_name( std::any::type_name::<A>() ) );
		Self{ id: mb.0, mb: mb.1 }
	}


	/// Automatically create a mailbox (thespis_impl::single_thread::Inbox) and an address from your
	/// actor. This avoids the boilerplate of manually having to create the mailbox and the address.
	/// Will consume your actor and return an address.
	///
	/// TODO: have doc examples tested by rustdoc
	///
	/// ```ignore
	/// let addr = Addr::try_from( MyActor{} )?;
	/// addr.call( MyMessage{} ).await?;
	/// ```
	//
	pub fn try_from( actor: A, exec: &mut impl Spawn ) -> ThesRes<Self> where A: Send
	{
		let inbox: Inbox<A> = Inbox::new()                ;
		let addr            = Self ::new( inbox.sender() );

		inbox.start( actor, exec )?;
		Ok( addr )
	}

	/// Automatically create a mailbox (thespis_impl::single_thread::Inbox) and an address from your
	/// actor. This avoids the boilerplate of manually having to create the mailbox and the address.
	/// Will consume your actor and return an address.
	///
	/// This will spawn the mailbox on the current thread. You need to set up async_runtime to enable
	/// spawn_local.
	///
	/// TODO: have doc examples tested by rustdoc
	///
	/// ```ignore
	/// let addr = Addr::try_from( MyActor{} )?;
	/// addr.call( MyMessage{} ).await?;
	/// ```
	//
	pub fn try_from_local( actor: A, exec: &mut impl LocalSpawn ) -> ThesRes<Self>
	{
		let inbox: Inbox<A> = Inbox::new()                ;
		let addr            = Self ::new( inbox.sender() );

		inbox.start_local( actor, exec )?;
		Ok( addr )
	}


	/// Get the id of the mailbox this address sends to. There will be exactly one for each
	/// actor, so you can use this for uniquely identifying your actors.
	///
	/// This is an atomic usize that is incremented for every new mailbox. There currently
	/// is no overflow protection.
	//
	pub fn id( &self ) -> usize
	{
		self.id
	}
}



// this doesn't work right now, because it would specialize a blanket impl from core...
// impl<A: Actor> TryFrom<A> for Addr<A>
// {
// 	type Error = Error;

// 	fn try_from( actor: A ) -> ThesRes<Self>
// 	{
// 		let inbox: Inbox<A> = Inbox::new();
// 		let addr = Self::new( inbox.sender() );

// 		inbox.start( actor )?;
// 		Ok( addr )
// 	}
// }


// For debugging
//
impl<A: Actor> Drop for Addr<A>
{
	fn drop( &mut self )
	{
		trace!( "DROP address for: {}", clean_name( std::any::type_name::<A>() ) );
	}
}



impl<A, M> Recipient<M> for Addr<A>

	where  A: Actor + Handler<M> ,
	       M: Message            ,

{
	fn call( &mut self, msg: M ) -> Return<'_, ThesRes< <M as Message>::Return >>
	{
		Box::pin( async move
		{
			let (ret_tx, ret_rx)     = oneshot::channel::<M::Return>()              ;
			let envl: BoxEnvelope<A> = Box::new( CallEnvelope::new( msg, ret_tx ) ) ;
			let result               = self.mb.send( envl ).await                   ;

			// MailboxClosed or MailboxFull
			//
			result.map_err( |e| Inbox::<A>::mb_error( e, format!("{:?}", self) ) )?;


			ret_rx.await

				.map_err( |_| ThesErr::MailboxClosedBeforeResponse{ actor: format!( "{:?}", self ) }.into() )
		})
	}



	fn clone_box( &self ) -> BoxRecipient<M, ThesErr>
	{
		Box::new( self.clone() )
	}


	fn actor_id( &self ) -> usize
	{
		self.id
	}
}



impl<A, M> Sink<M> for Addr<A>

	where A: Actor + Handler<M> ,
	      M: Message            ,

{
	type Error = ThesErr;

	fn poll_ready( self: Pin<&mut Self>, cx: &mut TaskContext<'_> ) -> Poll<Result<(), Self::Error>>
	{
		match self.mb.poll_ready( cx )
		{
			Poll::Ready( p ) => match p
			{
				Ok (_) => Poll::Ready( Ok(()) ),
				Err(e) =>
				{
					Poll::Ready( Err( Inbox::<A>::mb_error( e, format!("{:?}", self) ) ) )
				}
			}

			Poll::Pending => Poll::Pending
		}
	}


	fn start_send( mut self: Pin<&mut Self>, msg: M ) -> Result<(), Self::Error>
	{
		let envl: BoxEnvelope<A>= Box::new( SendEnvelope::new( msg ) );

		self.mb.start_send( envl ).map_err( |e| Inbox::<A>::mb_error( e, format!("{:?}", self) ))
	}


	fn poll_flush( mut self: Pin<&mut Self>, cx: &mut TaskContext<'_> ) -> Poll<Result<(), Self::Error>>
	{
		match Pin::new( &mut self.mb ).poll_flush( cx )
		{
			Poll::Ready( p ) => match p
			{
				Ok (_) => Poll::Ready( Ok(()) ),
				Err(e) =>
				{
					Poll::Ready( Err( Inbox::<A>::mb_error( e, format!("{:?}", self) )))
				}
			}

			Poll::Pending => Poll::Pending
		}
	}


	/// Will only close when dropped, this method can never return ready
	//
	fn poll_close( self: Pin<&mut Self>, _cx: &mut TaskContext<'_> ) -> Poll<Result<(), Self::Error>>
	{
		Poll::Pending
	}
}


impl<A, M> Address<A, M> for Addr<A>

	where  A: Actor + Handler<M>,
	       M: Message           ,

{
	fn recipient( &self ) -> BoxRecipient<M, ThesErr>
	{
		Box::new( self.clone() )
	}
}
