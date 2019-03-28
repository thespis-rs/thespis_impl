use crate :: { import::*, single_thread::* };


pub struct Addr< A: Actor + Send >
{
	_phantom: PhantomData< A >,
	mb      : mpsc::UnboundedSender<Box<dyn Envelope<A>>>,
}



impl< A> Address<A> for Addr<A>

	where A: Actor + Send + 'static,


{

	fn new( mb: mpsc::UnboundedSender<Box<dyn Envelope<A>>> ) -> Self
	{
		Self{ mb, _phantom: PhantomData }
	}



	fn send<M>( &mut self, msg: M ) -> Pin<Box< dyn Future<Output=()> + '_>>

		where A: Handler< M >,
		      M: Message<Result = ()> + Send + 'static,

	{
		async move
		{
			let envl: Box< dyn Envelope<A> + Send >= Box::new( SendEnvelope::new( msg ) );

			await!( self.mb.send( envl ) ).expect( "Failed to send to Mailbox" );

		}.boxed()
	}



	fn call<M: Message + 'static>( &mut self, msg: M ) -> Pin<Box< dyn Future< Output = M::Result > + Send + '_> >

		where A: Handler< M > ,
		      M: Send         ,
		      M::Result: Send,
		      A: Send         ,
	{
		async move
		{

			let (ret_tx, ret_rx) = oneshot::channel::<M::Result>();

			let envl: Box< dyn Envelope<A> + Send > = Box::new( CallEnvelope::new( msg, ret_tx ) );

			// trace!( "Sending envl to Mailbox" );

			await!( self.mb.send( envl ) ).expect( "Failed to send to Mailbox" );

			await!( ret_rx ).expect( "Failed to receive response in Addr.call" )

		}.boxed()
	}
}


