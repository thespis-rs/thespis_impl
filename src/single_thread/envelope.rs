use crate :: { import::*, single_thread::* };



impl<M> SendEnvelope<M> where M: Message
{
	pub fn new( msg: M ) -> Self
	{
		Self { msg }
	}
}

impl<A, M> Envelope<A> for SendEnvelope<M>

	where A: Actor,
	      M: Message,
	      A: Handler<M>,
{
	fn handle( self: Box<Self>, actor: &mut A ) -> TupleResponse
	{
		Box::pin( async move
		{
			let _ = await!( < A as Handler<M> >::handle( actor, self.msg ) );
		})
	}
}



pub struct CallEnvelope<M> where M: Message
{
	msg : M,
	addr: oneshot::Sender< M::Result >,
}

impl<M> CallEnvelope<M> where M: Message
{
	pub fn new( msg: M, addr: oneshot::Sender< M::Result > ) -> Self
	{
		Self { msg, addr }
	}
}

impl<A, M> Envelope<A> for CallEnvelope<M>

	where A: Actor,
	      M: Message,
	      A: Handler<M>,
{
	fn handle( self: Box<Self>, actor: &mut A ) -> TupleResponse
	{
		async move
		{
			let result = await!( < A as Handler<M> >::handle( actor, self.msg ) );

			// trace!( "Send from envelope" );

			match self.addr.send( result )
			{
				Ok (_) => {},
				Err(_) => { error!( "failed to send from envelope, receiving end dropped" ) },
			};

		}.boxed()
	}
}



pub struct SendEnvelope<M> where M: Message
{
	msg : M,
}
