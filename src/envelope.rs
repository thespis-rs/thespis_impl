use crate :: { import::* };





pub struct SendEnvelope<M> where M: Message
{
	msg : M,
}


impl<M> SendEnvelope<M> where M: Message
{
	pub fn new( msg: M ) -> Self
	{
		Self { msg }
	}
}

impl<A, M> Envelope<A> for SendEnvelope<M>

	where  A                     : Actor + Handler<M>,
	       M                     : Message           ,
{
	fn handle( self: Box<Self>, actor: &mut A ) -> Return<()>
	{
		Box::pin( async move
		{

			let _ = < A as Handler<M> >::handle( actor, self.msg ).await;

		})
	}
}



pub struct CallEnvelope<M> where M: Message
{
	msg : M,
	addr: oneshot::Sender< M::Return >,
}

impl<M> CallEnvelope<M> where M: Message
{
	pub fn new( msg: M, addr: oneshot::Sender< M::Return > ) -> Self
	{
		Self { msg, addr }
	}
}

impl<A, M> Envelope<A> for CallEnvelope<M>

	where A: Actor,
	      M: Message,
	      A: Handler<M>,
{
	fn handle( self: Box<Self>, actor: &mut A ) -> Return<()>
	{
		Box::pin( async move
		{
			let result = < A as Handler<M> >::handle( actor, self.msg ).await;

			// trace!( "Send from envelope" );

			match self.addr.send( result )
			{
				Ok (_) => {},
				Err(_) => { error!( "failed to send from envelope, receiving end dropped" ) },
			};

		})
	}
}

