use crate :: { import::*, *, mailbox::*, envelope::* };



/// Reference implementation of thespis::Address<A, M>.
/// It can receive all message types the actor implements thespis::Handler for.
/// An actor will be dropped when all addresses to it are dropped.
//
pub struct Addr< A: Actor >
{
	mb: mpsc::UnboundedSender< BoxEnvelope<A> >,
}



impl< A: Actor > Clone for Addr<A>
{
	fn clone( &self ) -> Self
	{
		trace!( "CREATE address for: {}", clean_name( unsafe{ std::intrinsics::type_name::<A>() } ) );

		Self { mb: self.mb.clone() }
	}
}


// TODO: test this and do we really want to introduce unsafe just for a type name?
//
impl<A: Actor> fmt::Debug for Addr<A>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		unsafe{ write!( f, "Addr<{:?}>", clean_name( std::intrinsics::type_name::<A>() ) ) }
	}
}


fn clean_name( name: &str ) -> String
{
	use regex::Regex;

	let re = Regex::new( r"\w+::" ).unwrap();

	let s = re.replace_all( name, "" );

	// this is just a specific one when using the Peer from remote
	//
	s.replace( "Peer<Compat01As03Sink<SplitSink<Framed<TcpStream, MulServTokioCodec<MultiServiceImpl<ServiceID, ConnID, Codecs>>>>, MultiServiceImpl<ServiceID, ConnID, Codecs>>, MultiServiceImpl<ServiceID, ConnID, Codecs>>", "Peer" )
}



impl<A> Addr<A> where A: Actor
{
	/// Create a new address. The simplest way is to use Addr::try_from( Actor ).
	/// This way allows more control. You need to manually make the mailbox. See the
	/// no_rt example in the repository.
	///
	// TODO: take a impl trait instead of a concrete type. This leaks impl details.
	//
	pub fn new( mb: mpsc::UnboundedSender<BoxEnvelope<A>> ) -> Self
	{
		trace!( "CREATE address for: {}", clean_name( unsafe{ std::intrinsics::type_name::<A>() } ) );
		Self{ mb }
	}


	/// Automatically create a mailbox (thespis_impl::single_thread::Inbox) and an address from your
	/// actor. This avoids the boilerplate of manually having to create the mailbox and the address.
	/// Will consume your actor and return an address.
	///
	/// TODO: have doc examples tested by rustdoc
	///
	/// ```ignore
	/// let addr = Addr::try_from( MyActor{} )?;
	/// await!( addr.call( MyMessage{} ) )?;
	/// ```
	//
	pub fn try_from( actor: A ) -> ThesRes<Self>
	{
		let inbox: Inbox<A> = Inbox::new();
		let addr = Self::new( inbox.sender() );

		inbox.start( actor )?;
		Ok( addr )
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
		trace!( "DROP address for: {}", clean_name( unsafe{ std::intrinsics::type_name::<A>() } ) );
	}
}



impl<A, M> Recipient<M> for Addr<A>

	where  A                     : Actor + Handler<M> ,
	       M                     : Message            ,

{
	fn sendr( &mut self, msg: M ) -> Return< ThesRes<()> >
	{
		async move
		{
			let envl: BoxEnvelope<A>= Box::new( SendEnvelope::new( msg ) );

			await!( self.mb.send( envl ) )?;
			// trace!( "sent envelope to mailbox" );

			Ok(())

		}.boxed()
	}



	fn call( &mut self, msg: M ) -> Return< ThesRes<<M as Message>::Return> >
	{
		async move
		{
			let (ret_tx, ret_rx) = oneshot::channel::<M::Return>();

			let envl: BoxEnvelope<A> = Box::new( CallEnvelope::new( msg, ret_tx ) );

			// trace!( "Sending envl to Mailbox in call" );

			await!( self.mb.send( envl ) )?;

			Ok( await!( ret_rx )? )

		}.boxed()
	}



	fn clone_box( &self ) -> BoxRecipient<M>
	{
		box self.clone()
	}
}



impl<A, M> Sink<M> for Addr<A>

	where A                     : Actor + Handler<M> ,
	      M                     : Message            ,

{
	type SinkError = Error;

	fn poll_ready( self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Result<(), Self::SinkError>>
	{
		match self.mb.poll_ready( cx )
		{
			Poll::Ready( p ) => match p
			{
				Ok (_) => Poll::Ready( Ok(())          ),
				Err(e) => Poll::Ready( Err( e.into() ) ),
			}

			Poll::Pending => Poll::Pending
		}
	}


	fn start_send( mut self: Pin<&mut Self>, msg: M ) -> Result<(), Self::SinkError>
	{
		let envl: BoxEnvelope<A>= Box::new( SendEnvelope::new( msg ) );

		self.mb.start_send( envl ).map_err( |e| e.into() )
	}


	fn poll_flush( self: Pin<&mut Self>, _cx: &mut Context ) -> Poll<Result<(), Self::SinkError>>
	{
		Poll::Ready(Ok(()))

		// The following does not compile, but it seems this method always responds as above on UnboundedSender
		//
		// match self.mb.poll_flush( cx )
		// {
		// 	Poll::Ready( p ) => match p
		// 	{
		// 		Ok (_) => Poll::Ready( Ok ( ()       ) ),
		// 		Err(e) => Poll::Ready( Err( e.into() ) ),
		// 	}

		// 	Poll::Pending => Poll::Pending
		// }
	}


	/// Will only close when dropped, this method can never return ready
	//
	fn poll_close( self: Pin<&mut Self>, _cx: &mut Context ) -> Poll<Result<(), Self::SinkError>>
	{
		Poll::Pending
	}
}


impl<A, M> Address<A, M> for Addr<A>

	where  A                     : Actor + Handler<M>,
	       M                     : Message           ,

{
	fn recipient( &self ) -> BoxRecipient<M>
	{
		box self.clone()
	}
}



/// This type can be used when you need a concrete type as Recipient<M>. Eg,
/// you can store this as BoxAny and then use down_cast from std::any::Any.
//
pub struct Receiver<M: Message>
{
	rec: Pin<BoxRecipient<M>>
}

impl<M: Message> Receiver<M>
{
	/// Create a new Receiver
	//
	pub fn new( rec: BoxRecipient<M> ) -> Self
	{
		Self { rec: Pin::from( rec ) }
	}
}



impl<M: Message> Clone for Receiver<M>
{
	fn clone( &self ) -> Self
	{
		Self { rec: Pin::from( self.rec.clone_box() ) }
	}
}


use std::ops::DerefMut;

impl<M: Message> Recipient<M> for Receiver<M>
{
	fn sendr( &mut self, msg: M ) -> Return< ThesRes<()> >
	{
		async move
		{

			await!( self.rec.deref_mut().sendr( msg ) )

		}.boxed()
	}



	fn call( &mut self, msg: M ) -> Return< ThesRes<<M as Message>::Return> >
	{
		async move
		{
			await!( self.rec.deref_mut().call( msg ) )

		}.boxed()
	}



	fn clone_box( &self ) -> BoxRecipient<M>
	{
		box self.clone()
	}
}



impl<M: Message> Sink<M> for Receiver<M>
{
	type SinkError = Error;

	fn poll_ready( mut self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Result<(), Self::SinkError>>
	{
		self.rec.as_mut().poll_ready( cx )
	}


	fn start_send( mut self: Pin<&mut Self>, msg: M ) -> Result<(), Self::SinkError>
	{
		self.rec.as_mut().start_send( msg )
	}


	fn poll_flush( mut self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Result<(), Self::SinkError>>
	{
		self.rec.as_mut().poll_flush( cx )
	}


	/// Will only close when dropped, this method can never return ready
	//
	fn poll_close( mut self: Pin<&mut Self>, cx: &mut Context ) -> Poll<Result<(), Self::SinkError>>
	{
		self.rec.as_mut().poll_close( cx )
	}
}
