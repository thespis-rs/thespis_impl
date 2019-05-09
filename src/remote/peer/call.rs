use { crate :: { import::*, runtime::rt, remote::{ *, error::* }, Addr, Receiver } };


/// Type representing the outgoing call. Used by a recipient to a remote service to communicate
/// an outgoing call to [Peer]. Also used by [Peer] to call a remote service when relaying.
///
/// MS must be of the same type as the type parameter on [Peer].
//
pub struct Call<MS: MultiService>
{
	mesg: MS,
}

impl<MS: 'static +  MultiService + Send> Message for Call<MS>
{
	type Return = Result< oneshot::Receiver<MS>, ThesRemoteErr >;
}

impl<MS: MultiService> Call<MS>
{
	pub fn new( mesg: MS ) -> Self
	{
		Self{ mesg }
	}
}



/// Handler for outgoing Calls
//
// we use channels to create an async response.
//
impl<Out, MS> Handler<Call<MS>> for Peer<Out, MS>

	where Out: BoundsOut<MS>,
	      MS : BoundsMS     ,
{
	fn handle( &mut self, call: Call<MS> ) -> ReturnNoSend< <Call<MS> as Message>::Return >
	{
		trace!( "peer: starting Handler<Call<MS>>" );

		Box::pin( async move
		{
			// Fallible operations first
			//
			let conn_id = call.mesg.conn_id()?;
			await!( self.send_msg( call.mesg ) )?;

			// If the above succeeded, store the other end of the channel
			//
			let (sender, receiver) = oneshot::channel::< MS >() ;

			self.responses.insert( conn_id, sender );

			Ok( receiver )

		})
	}
}
