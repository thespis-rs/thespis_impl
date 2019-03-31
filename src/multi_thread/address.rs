use crate :: { import::*, multi_thread::* };


pub struct Addr< A: Actor >
{
	mb: mpsc::UnboundedSender<Box<dyn ThreadSafeEnvelope<A>>>,
}

impl< A: Actor > Clone for Addr<A>
{
	fn clone( &self ) -> Self
	{
		Self { mb: self.mb.clone() }
	}
}

impl<A> Addr<A> where A: Actor + 'static
{
	// TODO: take a impl trait instead of a concrete type. This can be fixed once we
	// ditch channels or write some channels that implement sink.
	//
	pub fn new( mb: mpsc::UnboundedSender<Box<dyn ThreadSafeEnvelope<A>>> ) -> Self
	{
		Self{ mb }
	}
}

impl<A> ThreadSafeAddress<A> for Addr<A>

	where A: Actor + 'static,

{
	fn send<M>( &mut self, msg: M ) -> ThreadSafeTupleResponse

		where A: Handler< M >,
		      M: ThreadSafeMessage<Result = ()> + 'static,
		      <M as Message>::Result: Send,

	{
		async move
		{
			let envl: Box< dyn ThreadSafeEnvelope<A> >= Box::new( SendEnvelope::new( msg ) );

			await!( self.mb.send( envl ) ).expect( "Failed to send to Mailbox" );

		}.boxed()
	}



	fn call<M>( &mut self, msg: M ) -> Pin<Box< dyn Future< Output = <M as Message>::Result > + Send >>

		where A: Handler< M > ,
		      M: ThreadSafeMessage + 'static,
		      <M as Message>::Result: Send,

	{
		let mut mb = self.mb.clone();

		async move
		{
			let (ret_tx, ret_rx) = oneshot::channel::< <M as Message>::Result >();

			let envl: Box< dyn ThreadSafeEnvelope<A> > = Box::new( CallEnvelope::new( msg, ret_tx ) );

			await!( mb.send( envl ) ).expect( "Failed to send to Mailbox"               );
			await!( ret_rx          ).expect( "Failed to receive response in Addr.call" )

		}.boxed()
	}



	fn recipient<M>( &self ) -> Box< dyn ThreadSafeRecipient<M> >

		where M: ThreadSafeMessage + 'static,
		      A: Handler<M> + 'static,
		      <M as Message>::Result: Send,

	{
		box Receiver{ addr: self.clone() }
	}
}



struct Receiver<A: Actor + 'static>
{
	addr: Addr<A>
}



impl<A, M> ThreadSafeRecipient<M> for Receiver<A>

	where A: Handler< M > + 'static,
	      M: ThreadSafeMessage + 'static,
	      <M as Message>::Result: Send,

{
	default fn send( &mut self, msg: M ) -> ThreadSafeTupleResponse

		where M: ThreadSafeMessage<Result = ()>,
	{
		self.addr.send( msg )
	}



	default fn call( &mut self, msg: M ) -> ThreadSafeResponse<M>

	where A: Handler< M > ,
	      M: ThreadSafeMessage,
	      <M as Message>::Result: Send,

	{
		self.addr.call( msg )
	}
}
