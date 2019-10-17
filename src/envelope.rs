use crate::{import::*};



/// An envelope for sending a message to an actor ignoring the return value.
//
pub(crate) struct SendEnvelope<M> where M: Message
{
	msg: M,
}



impl<M> SendEnvelope<M> where M: Message
{
	pub(crate) fn new( msg: M ) -> Self
	{
		Self { msg }
	}
}



impl<A, M> Envelope<A> for SendEnvelope<M>

	where  A: Actor + Handler<M>,
			 M: Message           ,

{
	fn handle( self: Box<Self>, actor: &mut A ) -> Return<'_, ()> where <M as Message>::Return: Send,
	{
		< A as Handler<M> >::handle( actor, self.msg )

			.map( |_|() )
			.boxed()
	}

	fn handle_local( self: Box<Self>, actor: &mut A ) -> ReturnNoSend<'_, ()>
	{
		< A as Handler<M> >::handle_local( actor, self.msg )

			.map( |_|() )
			.boxed_local()
	}
}



/// An envelope that will take care of returning the result of the handle method.
//
pub(crate) struct CallEnvelope<M> where M: Message
{
	msg : M,
	addr: oneshot::Sender< M::Return >,
}



impl<M> CallEnvelope<M> where M: Message
{
	pub(crate) fn new( msg: M, addr: oneshot::Sender< M::Return > ) -> Self
	{
		Self { msg, addr }
	}
}



impl<A, M> Envelope<A> for CallEnvelope<M>

	where A: Actor      ,
	      M: Message    ,
	      A: Handler<M> ,
{
	fn handle( self: Box<Self>, actor: &mut A ) -> Return<'_, ()>
	{
		let CallEnvelope { msg, addr } = *self;

		let fut = < A as Handler<M> >::handle( actor, msg );


		async
		{
			// trace!( "Send from envelope" );

			// Send the result back to the address.
			//
			if addr.send( fut.await ).is_err()
			{
				debug!( "failed to send from envelope, Addr already dropped?" );
			}

		}.boxed()
	}


	fn handle_local( self: Box<Self>, actor: &mut A ) -> ReturnNoSend<'_, ()>
	{
		let CallEnvelope { msg, addr } = *self;

		let fut = < A as Handler<M> >::handle_local( actor, msg );


		async
		{
			// trace!( "Send from envelope" );

			// Send the result back to the address.
			//
			if addr.send( fut.await ).is_err()
			{
				debug!( "failed to send from envelope, Addr already dropped?" );
			}

		}.boxed_local()
	}
}

