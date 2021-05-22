use crate::{ import::*, ActorBuilder, ChanSender, StrongCount, WeakAddr, addr_inner::*, error::* };


/// Reference implementation of `thespis::Address<M>`.
/// It can be used to send all message types the actor implements thespis::Handler for.
/// An actor will be dropped when all addresses to it are dropped.
//
pub struct Addr< A: Actor >
{
	inner: AddrInner<A> ,
}


impl< A: Actor > Clone for Addr<A>
{
	fn clone( &self ) -> Self
	{
		trace!( "CREATE (clone) Addr for: {}", self );

		self.inner.strong.lock().expect( "Mutex<StrongCount> poisoned" ).increment();

		Self
		{
			inner: self.inner.clone() ,
		}
	}
}


/// Verify whether 2 Receivers will deliver to the same actor.
//
impl< A: Actor > PartialEq for Addr<A>
{
	fn eq( &self, other: &Self ) -> bool
	{
		self.inner == other.inner
	}
}

impl< A: Actor > Eq for Addr<A>{}



impl<A: Actor> fmt::Debug for Addr<A>
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
			"Addr<{}> ~ {}{}"          ,
			std::any::type_name::<A>() ,
			&self.id()                 ,
			name                       ,
		)
	}
}



impl<A: Actor> fmt::Display for Addr<A>
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		let name = std::any::type_name::<A>();
		let t = match name.split( "::" ).last()
		{
			Some(t) => t,
			None    => name,
		};

		match &self.name()
		{
			Some(n) => write!( f, "{} ({}, {})", t, self.id(), n ) ,
			None    => write!( f, "{} ({})"    , t, self.id()    ) ,
		}
	}
}




impl<A> Addr<A> where A: Actor
{
	// Create a new address. This is restricted to the crate because StrongCount does not
	// distinguish between the initial state (count==0) and the final (mailbox closed).
	//
	// If we allowed users to call this directly, they could in principle re-create strong
	// addresses after the mailbox has closed. Now the only way to make your first Addr is
	// through [`Mailbox::addr`](crate::Mailbox::addr).
	//
	pub(crate) fn new( id: usize, name: Option< Arc<str> >, tx: ChanSender<A>, strong: Arc<Mutex<StrongCount>> ) -> Self
	{
		strong.lock().expect( "Mutex<StrongCount> poisoned" ).increment();

		let inner = AddrInner::new( id, name, tx, strong );
		let this  = Self{ inner };

		trace!( "CREATE Addr for: {}", &this );

		this
	}


	/// Produces a builder for convenient creation of both [`Addr`] and [`Mailbox`](crate::Mailbox).
	//
	pub fn builder() -> ActorBuilder<A>
	{
		Default::default()
	}


	/// Create a new WeakAddr.
	//
	pub fn weak( &self ) -> WeakAddr<A>
	{
		WeakAddr::from( self.inner.clone() )
	}
}



impl<A: Actor> Drop for Addr<A>
{
	fn drop( &mut self )
	{
		trace!( "DROP Addr for: {}", self );

		self.inner.strong.lock().expect( "Mutex<StrongCount> poisoned" ).decrement();
	}
}



impl<A, M> Address<M> for Addr<A>

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
		self.inner.id()
	}

	fn name( &self ) -> Option< Arc<str> >
	{
		self.inner.name()
	}
}



impl<A, M> Sink<M> for Addr<A>

	where A: Actor + Handler<M> ,
	      M: Message            ,

{
	type Error = ThesErr;

	fn poll_ready( mut self: Pin<&mut Self>, cx: &mut TaskContext<'_> ) -> Poll<Result<(), Self::Error>>
	{
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


impl<A: Actor> TryFrom< AddrInner<A> > for Addr<A>
{
	type Error = ThesErr;

	fn try_from( inner: AddrInner<A> ) -> Result< Self, ThesErr >
	{
		let strong = inner.strong.lock().expect( "Mutex<StrongCount> poisoned" );

		// If already zero, we don't allow making a new strong address.
		//
		if strong.count() == 0
		{
			Err( ThesErr::MailboxClosed{ actor: format!("{:?}", &inner) } )
		}

		else
		{
			strong.increment();
			drop(strong);

			Ok( Self{ inner } )
		}
	}
}
