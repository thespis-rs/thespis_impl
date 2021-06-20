use crate::import::*;


/// Wrapper for a message that is generic over actor instead of over message type.
/// This allows sending messages of different types over the same channel.
///
/// It knows how to call handle on a given actor of the correct type.
//
pub trait Envelope<A> where A: Actor, Self: Send
{
	/// Have the actor handle the message
	//
	#[ must_use = "Futures do nothing unless polled" ]
	//
	fn handle( self: Box<Self>, actor: &mut A ) -> Return<'_, ()> where A: Send;


	/// Have the actor handle the message, for !Send Actors.
	//
	#[ must_use = "Futures do nothing unless polled" ]
	//
	fn handle_local( self: Box<Self>, actor: &mut A ) -> ReturnNoSend<'_, ()>;
}


/// An envelope for sending a message to an actor ignoring the return value.
//
pub(crate) struct SendEnvelope<M> where M: Message
{
	// must remain private because of unsafe Sync impl.
	//
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
	fn handle( self: Box<Self>, actor: &mut A ) -> Return<'_, ()>
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
	// must remain private because of unsafe Sync impl.
	//
	msg : M,
	addr: OneSender< M::Return >,
}



impl<M> CallEnvelope<M> where M: Message
{
	pub(crate) fn new( msg: M, addr: OneSender< M::Return > ) -> Self
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
		let CallEnvelope { msg, mut addr } = *self;

		let fut = < A as Handler<M> >::handle( actor, msg );


		async move
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
		let CallEnvelope { msg, mut addr } = *self;

		let fut = < A as Handler<M> >::handle_local( actor, msg );


		async move
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

