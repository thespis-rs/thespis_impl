use crate :: { import::*, * };


pub struct ProcLocalAddr< A: Actor + Send >
{
	_phantom: PhantomData< A >,
	mb      : mpsc::UnboundedSender<Box<dyn Envelope<A>>>,
}


impl<A: Actor + Send> ProcLocalAddr<A>
{

}



impl< A> Address<A> for ProcLocalAddr<A>

	where A: Actor + Send,


{

	fn new( mb: mpsc::UnboundedSender<Box<dyn Envelope<A>>> ) -> Self
	{
		Self{ mb, _phantom: PhantomData }
	}



	fn send<M>( &mut self, _msg: M )

		where A: Handler< M >,
		      M: Message<Result = ()> ,

	{

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

			// let ret : Result<Pin<Box< dyn Future< Output = M::Result > + Send >>, oneshot::Canceled>= ;

			await!( ret_rx ).expect( "Failed to receive response in ProcLocalAddr.call" )
		})
	}
}


