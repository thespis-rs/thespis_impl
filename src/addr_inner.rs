use crate::{ import::*, ChanSender, BoxEnvelope, StrongCount, envelope::*, error::* };



// Reference implementation of `thespis::Address<M>`.
// It can be used to send all message types the actor implements `thespis::Handler` for.
// An actor will be dropped when all strong addresses (`Addr`) to it are dropped. `WeakAddr` will
// not keep the mailbox+actor alive.
//
// We need the strong count in a mutex because sometimes we need to check the value and then modify
// it, being sure no other thread is messing with it at the same time. Atomic counter doesn't suffice.
//
pub(crate) struct AddrInner< A: Actor >
{
	           mb    : ChanSender<A>             ,
	           id    : usize                     ,
	           name  : Option< Arc<str> >        ,
	pub(crate) strong: Arc<Mutex< StrongCount >> ,
}



impl< A: Actor > Clone for AddrInner<A>
{
	fn clone( &self ) -> Self
	{
		Self
		{
			mb    : self.mb.clone_sink() ,
			id    : self.id              ,
			name  : self.name.clone()    ,
			strong: self.strong.clone()  ,
		}
	}
}


/// Verify whether 2 Receivers will deliver to the same actor.
//
impl< A: Actor > PartialEq for AddrInner<A>
{
	fn eq( &self, other: &Self ) -> bool
	{
		self.id == other.id
	}
}

impl< A: Actor > Eq for AddrInner<A>{}



impl<A: Actor> fmt::Debug for AddrInner<A>
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
			"AddrInner<{}> ~ {}{}"     ,
			std::any::type_name::<A>() ,
			&self.id                   ,
			name                       ,
		)
	}
}




impl<A> AddrInner<A> where A: Actor
{
	/// Create a new address. The simplest way is to use Addr::try_from( Actor ).
	/// This way allows more control. You need to manually make the mailbox. See the
	/// no_rt example in the repository.
	//
	pub(crate) fn new( id: usize, name: Option< Arc<str> >, tx: ChanSender<A>, strong: Arc<Mutex<StrongCount>> ) -> Self
	{
		Self{ id, name, mb: tx, strong }
	}



	/// Obtain a [`tracing::Span`] identifying the actor with it's id and it's name if it has one.
	//
	pub(crate) fn span( &self ) -> Span
	{
		if let Some( name ) = &self.name
		{
			error_span!( "actor", id = self.id, "type" = self.type_name(), name = name.as_ref() )
		}

		else
		{
			error_span!( "actor", id = self.id, "type" = self.type_name() )
		}
	}


	/// The type of the actor.
	//
	pub(crate) fn type_name( &self ) -> &str
	{
		let name = std::any::type_name::<A>();

		match name.split( "::" ).last()
		{
			Some(t) => t,
			None    => name,
		}
	}


	pub(crate) fn actor_info( &self ) -> String
	{
		if let Some(name) = &self.name
		{
			format!("{}: id={}, name={}", self.type_name(), self.id, name)
		}

		else
		{
			format!("{}: id={}", self.type_name(), self.id)
		}
	}
}




impl<A, M> Address<M> for AddrInner<A>

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

			// MailboxClosed - either the actor panicked, or all strong addresses to the mb
			// were dropped.
			//
			result.map_err( |_| ThesErr::MailboxClosed{ actor: self.actor_info() } )?;


			// We have a call type message. It was successfully delivered to the mailbox,
			// but the actor crashed before it sent us back a response.
			//
			ret_rx.await

				.map_err( |_| ThesErr::ActorStoppedBeforeResponse{ actor: self.actor_info() } )

		}.boxed()
	}



	fn clone_box( &self ) -> BoxAddress<M, ThesErr>
	{
		Box::new( self.clone() )
	}
}


impl<A> Identify for AddrInner<A>

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



impl<A, M> Sink<M> for AddrInner<A>

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
					Poll::Ready( Err( ThesErr::MailboxClosed{ actor: self.actor_info() } ) )
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
			.map_err( |_| ThesErr::MailboxClosed{ actor: self.actor_info() } )
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
					Poll::Ready( Err( ThesErr::MailboxClosed{ actor: self.actor_info() } ))
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
