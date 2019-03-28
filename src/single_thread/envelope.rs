use crate :: { import::*, single_thread::* };

pub struct CallEnvelope<M> where M: Message + Send + 'static
{
	msg : M,
	addr: oneshot::Sender< M::Result >,
}

impl<M> CallEnvelope<M> where M: Message + Send
{
	pub fn new( msg: M, addr: oneshot::Sender< M::Result > ) -> Self
	{
		Self { msg, addr }
	}
}

impl<A, M> Envelope<A> for CallEnvelope<M>

	where A: Actor + Send,
	      M: Message + Send,
	      M::Result: Send,
	      A: Handler<M>,
{
	fn handle( self: Box<Self>, actor: &mut A ) -> Pin<Box< dyn Future< Output = () > + Send + '_> >
	{
		Box::pin( async move
		{
			let result = await!( < A as Handler<M> >::handle( actor, self.msg ) );

			// trace!( "Send from envelope" );

			match self.addr.send( result )
			{
				Ok (_) => {},
				Err(_) => { error!( "failed to send from envelope, receiving end dropped" ) },
			};
		})
	}
}



pub struct SendEnvelope<M> where M: Message + Send + 'static
{
	msg : M,
}

impl<M> SendEnvelope<M> where M: Message + Send
{
	pub fn new( msg: M ) -> Self
	{
		Self { msg }
	}
}

impl<A, M> Envelope<A> for SendEnvelope<M>

	where A: Actor + Send,
	      M: Message + Send,
	      M::Result: Send,
	      A: Handler<M>,
{
	fn handle( self: Box<Self>, actor: &mut A ) -> Pin<Box< dyn Future< Output = () > + Send + '_> >
	{
		Box::pin( async move
		{
			let _ = await!( < A as Handler<M> >::handle( actor, self.msg ) );
		})
	}
}
