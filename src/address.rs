use crate :: { import::*, * };


pub struct ProcLocalAddr< A: Actor + Send >
{
	_phantom: PhantomData< A >,
	mb      : mpsc::UnboundedSender<Box<dyn Envelope<A>>>,
}



impl< A> Address<A> for ProcLocalAddr<A>

	where A: Actor + Send + 'static,


{

	fn new( mb: mpsc::UnboundedSender<Box<dyn Envelope<A>>> ) -> Self
	{
		Self{ mb, _phantom: PhantomData }
	}



	fn send<M>( &mut self, msg: M )

		where A: Handler< M >,
		      M: Message<Result = ()> + Send + 'static,

	{
		let envl: Box< dyn Envelope<A> + Send >= Box::new( SendEnvelope::new( msg ) );

		let mut mbb = self.mb.clone();

		let fut = async move
		{
			await!( mbb.send( envl ) ).expect( "Failed to send to mailbox" );
		};

		let mut executor = ThreadPool::new().unwrap();

		executor.spawn( fut ).unwrap();
	}



	fn call<M: Message + 'static>( &mut self, msg: M ) -> Pin<Box< dyn Future< Output = M::Result > + Send + '_> >

		where A: Handler< M > ,
		      M: Send         ,
		      M::Result: Send,
		      A: Send         ,
	{
		Box::pin( async move {

			let (ret_tx, ret_rx) = oneshot::channel::<M::Result>();

			let envl: Box< dyn Envelope<A> + Send >= Box::new( ChannelEnvelope::new( msg, ret_tx ) );

			trace!( "Sending envl to mailbox" );

			await!( self.mb.send( envl ) ).expect( "Failed to send to mailbox" );

			await!( ret_rx ).expect( "Failed to receive response in ProcLocalAddr.call" )
		})
	}
}


