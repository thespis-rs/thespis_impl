use crate::{ import::*, remote::* };

#[ derive( Debug, Clone, PartialEq ) ]
//
pub enum PeerEvent
{
	Closed                   ,
	ClosedByRemote           ,
	Error( ConnectionError ) ,
}


impl Message for PeerEvent
{
	type Return = ();
}



/// Handler for peer events from other peers, mainly for peers that we have to relay over.
//
impl<Out, MS> Handler<PeerEvent> for Peer<Out, MS>

	where Out: BoundsOut<MS>,
	      MS : BoundsMS     ,
{
	fn handle( &mut self, _evt: PeerEvent ) -> ReturnNoSend< <PeerEvent as Message>::Return >
	{
		trace!( "peer: starting Handler<PeerEvent>" );

		Box::pin( async move
		{

		})
	}
}
