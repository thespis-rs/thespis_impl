use { crate :: { import::*, ThesError, runtime::rt, remote::*, single_thread::{ Addr, Rcpnt } } };


/// Type representing the outgoing call. Used by a recipient to a remote service to communicate
/// an outgoing call to [Peer]. Also used by [Peer] to call a remote service when relaying.
///
/// MulService must be of the same type as the type parameter on [Peer].
//
pub struct Call<MulService: MultiService>
{
	mesg: MulService,
}

impl<MulService: 'static +  MultiService> Message for Call<MulService>
{
	type Result = ThesRes< oneshot::Receiver<MulService> >;
}

impl<MulService: MultiService> Call<MulService>
{
	pub fn new( mesg: MulService ) -> Self
	{
		Self{ mesg }
	}
}



/// Handler for outgoing Calls
//
// we use channels to create an async response.
//
impl<Out, MulService> Handler<Call<MulService>> for Peer<Out, MulService>

	where Out       : BoundsOut<MulService>,
	      MulService: BoundsMulService     ,
{
	fn handle( &mut self, call: Call<MulService> ) -> Return< <Call<MulService> as Message>::Result >
	{
		trace!( "peer: starting Handler<Call<MulService>>" );

		async move
		{
			let (sender, receiver) = oneshot::channel::< MulService >() ;
			let conn_id            = call.mesg.conn_id()?               ;

			self.responses.insert( conn_id, sender );

			await!( self.send_msg( call.mesg ) )?;

			Ok( receiver )

		}.boxed()
	}
}
