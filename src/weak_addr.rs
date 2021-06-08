use crate::{ import::*, Addr, addr_inner::*, error::* };


/// Reference implementation of `thespis::Address<M>`.
/// It can be used to send all message types the actor implements thespis::Handler for.
/// An actor will be dropped when all addresses to it are dropped.
//
pub struct WeakAddr< A: Actor >
{
	inner: AddrInner<A>,
}


impl< A: Actor > Clone for WeakAddr<A>
{
	fn clone( &self ) -> Self
	{
		let _s = self.span().entered();
		trace!( "CREATE WeakAddr" );

		Self
		{
			inner: self.inner.clone(),
		}
	}
}


/// Verify whether 2 Receivers will deliver to the same actor.
//
impl< A: Actor > PartialEq for WeakAddr<A>
{
	fn eq( &self, other: &Self ) -> bool
	{
		self.inner == other.inner
	}
}

impl< A: Actor > Eq for WeakAddr<A>{}



impl<A: Actor> fmt::Debug for WeakAddr<A>
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		let name = match &self.name()
		{
			Some( s ) => format!( ", {}", s ) ,
			None      => String::new()        ,
		};

		write!
		(
			f                          ,
			"WeakAddr<{}> ~ {}{}"      ,
			std::any::type_name::<A>() ,
			&self.id()                 ,
			name                       ,
		)
	}
}



impl<A: Actor> fmt::Display for WeakAddr<A>
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		match &self.name()
		{
			Some(n) => write!( f, "{} ({}, {})", self.inner.type_name(), self.id(), n ) ,
			None    => write!( f, "{} ({})"    , self.inner.type_name(), self.id()    ) ,
		}
	}
}




impl<A> WeakAddr<A> where A: Actor
{
	/// Create a strong address. This requires that there are still other
	/// strong addresses around at the time of this call, otherwise this will
	/// return [`ThesErr::MailboxClosed`].
	//
	pub fn strong( &self ) -> Result< Addr<A>, ThesErr >
	{
		Addr::try_from( self.inner.clone() )
	}


	/// Obtain a [`tracing::Span`] identifying the actor with it's id and it's name if it has one.
	//
	pub fn span( &self ) -> Span
	{
		self.inner.span()
	}
}

// For debugging
//
impl<A: Actor> Drop for WeakAddr<A>
{
	fn drop( &mut self )
	{
		let _s = self.span().entered();
		trace!( "DROP WeakAddr" );
	}
}



impl<A, M> Address<M> for WeakAddr<A>

	where  A: Actor + Handler<M> ,
	       M: Message            ,

{
	fn call( &mut self, msg: M ) -> Return<'_, ThesRes< <M as Message>::Return >>
	{
		self.inner.call( msg )
	}



	fn clone_box( &self ) -> BoxAddress<M, ThesErr>
	{
		Box::new( self.clone() )
	}
}


impl<A> Identify for WeakAddr<A>

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
		self.inner.id()
	}

	fn name( &self ) -> Option< Arc<str> >
	{
		self.inner.name()
	}
}



impl<A, M> Sink<M> for WeakAddr<A>

	where A: Actor + Handler<M> ,
	      M: Message            ,

{
	type Error = ThesErr;

	fn poll_ready( mut self: Pin<&mut Self>, cx: &mut TaskContext<'_> ) -> Poll<Result<(), Self::Error>>
	{
		// There are no more strong addresses around, we no longer accept messages.
		//
		if self.inner.strong.lock().expect( "Mutex<StrongCount> poisoned" ).count() == 0
		{
			return Poll::Ready( Err( ThesErr::MailboxClosed{ actor: self.inner.actor_info() } ) )
		}

		Pin::new( &mut self.inner ).poll_ready( cx )
	}


	fn start_send( mut self: Pin<&mut Self>, msg: M ) -> Result<(), Self::Error>
	{
		Pin::new( &mut self.inner ).start_send( msg )
	}


	fn poll_flush( mut self: Pin<&mut Self>, cx: &mut TaskContext<'_> ) -> Poll<Result<(), Self::Error>>
	{
		Pin::new( &mut self.inner ).poll_flush( cx )
	}


	/// This is a no-op. The address can only really close when dropped. Close has no meaning before that.
	//
	fn poll_close( mut self: Pin<&mut Self>, cx: &mut TaskContext<'_> ) -> Poll<Result<(), Self::Error>>
	{
		Pin::new( &mut self.inner ).poll_flush( cx )
	}
}


impl<A: Actor> From< AddrInner<A> > for WeakAddr<A>
{
	fn from( inner: AddrInner<A> ) -> Self
	{
		Self{ inner }
	}
}
