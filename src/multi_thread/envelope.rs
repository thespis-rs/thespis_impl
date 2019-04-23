use crate :: { import::*, multi_thread::* };



impl<M> SendEnvelope<M> where M: Message
{
	pub fn new( msg: M ) -> Self
	{
		Self { msg }
	}
}

impl<A, M> Envelope<A> for SendEnvelope<M>

	where  A                    : Actor          ,
	       A                    : Handler<M>     ,
	       M                    : Message + Send ,
	      <M as Message>::Return: Send           ,
{
	fn handle( self: Box<Self>, actor: &mut A ) -> Return<()>
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
	      M: Message + Send,
	      A: Handler<M>,
	      <M as Message>::Return: Send,
{
	fn handle( self: Box<Self>, actor: &mut A ) -> Return<()>
	{
		async move
		{
			trace!( "Await Actor.handle result" );

			let result = await!( < A as Handler<M> >::handle( actor, self.msg ) );

			trace!( "Send from envelope" );

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
