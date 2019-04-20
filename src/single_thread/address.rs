use crate :: { import::*, single_thread::* };


/// Reference implementation of thespis::Address.
/// It can receive all message types the actor implements thespis::Handler for.
//
pub struct Addr< A: Actor >
{
	mb: mpsc::UnboundedSender<Box<dyn Envelope<A>>>,
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



impl<A> Addr<A> where A: Actor
{
	// TODO: take a impl trait instead of a concrete type. This can be fixed once we
	// ditch channels or write some channels that implement sink.
	//
	pub fn new( mb: mpsc::UnboundedSender<Box< dyn Envelope<A> >> ) -> Self
	{
		trace!( "CREATE address for: {}", clean_name( unsafe{ std::intrinsics::type_name::<A>() } ) );
		Self{ mb }
	}


	/// Automatically create a mailbox (thespis_impl::single_thread::Inbox) and an address from your
	/// actor. This avoids the boilerplate of manually having to create the mailbox and the address.
	/// Will consume your actor and return an address.
	///
	/// Since this is a convenience method, it avoids returning a result, however it spawns the
	/// mailbox which in theory can fail, so this can panic.
	///
	/// ```ignore
	/// let addr = Addr::from( MyActor{} );
	/// await!( addr.call( MyMessage{} ) )?;
	/// ```
	//
	pub fn from( actor: A ) -> Self
	{
		let inbox: Inbox<A> = Inbox::new();
		let addr = Self::new( inbox.sender() );

		inbox.start( actor ).expect( "spawn inbox" );
		addr
	}
}



fn clean_name( name: &str ) -> String
{
	use regex::Regex;

	let re = Regex::new( r"\w+::" ).unwrap();

	let s = re.replace_all( name, "" );

	s.replace( "Peer<Compat01As03Sink<SplitSink<Framed<TcpStream, MulServTokioCodec<MultiServiceImpl<ServiceID, ConnID, Codecs>>>>, MultiServiceImpl<ServiceID, ConnID, Codecs>>, MultiServiceImpl<ServiceID, ConnID, Codecs>>", "Peer" )
}


impl<A: Actor> Drop for Addr<A>
{
	fn drop( &mut self )
	{
		trace!( "DROP address for: {}", clean_name( unsafe{ std::intrinsics::type_name::<A>() } ) );
	}
}



impl<A> Address<A> for Addr<A>

	where A: Actor,

{
	fn send<M>( &mut self, msg: M ) -> Response< ThesRes<()> >

		where A: Handler< M >,
		      M: Message<Result = ()>,

	{
		async move
		{
			let envl: Box< dyn Envelope<A> >= Box::new( SendEnvelope::new( msg ) );

			await!( self.mb.send( envl ) )?;
			trace!( "sent to mailbox" );


			Ok(())

		}.boxed()
	}



	fn call<M: Message>( &mut self, msg: M ) -> Response< ThesRes<<M as Message>::Result> >

		where A: Handler< M > ,

	{
		async move
		{
			let (ret_tx, ret_rx) = oneshot::channel::<M::Result>();

			let envl: Box< dyn Envelope<A> > = Box::new( CallEnvelope::new( msg, ret_tx ) );

			// trace!( "Sending envl to Mailbox" );

			await!( self.mb.send( envl ) )?;
			trace!( "call to mailbox" );

			Ok( await!( ret_rx )? )

		}.boxed()
	}



	fn recipient<M>( &self ) -> Box< dyn Recipient<M> > where M: Message, A: Handler<M>
	{
		box Receiver{ addr: self.clone() }
	}
}




struct Receiver<A: Actor>
{
	addr: Addr<A>
}


impl<A: Actor> Clone for Receiver<A>
{
	fn clone( &self ) -> Self
	{
		Self { addr: self.addr.clone() }
	}
}



pub struct Rcpnt<M: Message>
{
	rec: Box< dyn Recipient<M> >
}

impl<M: Message> Rcpnt<M>
{
	pub fn new( rec: Box< dyn Recipient<M> > ) -> Self
	{
		Self { rec }
	}
}



impl<M: Message> Recipient<M> for Rcpnt<M>
{
	fn send( &mut self, msg: M ) -> Response< ThesRes<()> >

		where M: Message<Result = ()>,
	{
		async move
		{
			await!( self.rec.send( msg ) )

		}.boxed()
	}



	fn call( &mut self, msg: M ) -> Response< ThesRes<<M as Message>::Result> >
	{
		async move
		{
			await!( self.rec.call( msg ) )

		}.boxed()
	}

	fn clone_box( &self ) -> Box< dyn Recipient<M> >
	{
		box Self { rec: self.rec.clone_box() }
	}

}


impl<A, M> Recipient<M> for Receiver<A>

	where A: Handler<M>  ,
	      M: Message     ,

{
	default fn send( &mut self, msg: M ) -> Response< ThesRes<()> >

		where M: Message<Result = ()>,
	{
		self.addr.send( msg )
	}



	default fn call( &mut self, msg: M ) -> Response< ThesRes<<M as Message>::Result> >
	{
		self.addr.call( msg )
	}



	fn clone_box( &self ) -> Box< dyn Recipient<M> >
	{
		box Self { addr: self.addr.clone() }
	}
}





