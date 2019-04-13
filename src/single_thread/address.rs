use crate :: { import::*, single_thread::* };


pub struct Addr< A: Actor >
{
	mb: mpsc::UnboundedSender<Box<dyn Envelope<A>>>,
}

impl< A: Actor > Clone for Addr<A>
{
	fn clone( &self ) -> Self
	{
		Self { mb: self.mb.clone() }
	}
}

impl<A> Addr<A> where A: Actor
{
	// TODO: take a impl trait instead of a concrete type. This can be fixed once we
	// ditch channels or write some channels that implement sink.
	//
	pub fn new( mb: mpsc::UnboundedSender<Box< dyn Envelope<A> >> ) -> Self
	{
		Self{ mb }
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



impl<A, M> Recipient<M> for Receiver<A>

	where A: Handler<M>            ,
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
}





