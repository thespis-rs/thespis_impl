use crate::{ import::*, remote::* };

#[ derive( Debug, Clone, PartialEq ) ]
//
pub enum PeerEvent
{
	Closed                   ,
	ClosedByRemote           ,
	RelayDisappeared(usize)  ,
	Error( ConnectionError ) ,
}


#[ derive( Debug, Clone ) ]
//
pub(super) struct RelayEvent
{
	pub id : usize    ,
	pub evt: PeerEvent,
}


impl Message for RelayEvent
{
	type Return = ();
}



/// Handler for peer events from other peers, mainly for peers that we have to relay over.
//
impl<Out, MS> Handler<RelayEvent> for Peer<Out, MS>

	where Out: BoundsOut<MS>,
	      MS : BoundsMS     ,
{
	fn handle( &mut self, re: RelayEvent ) -> ReturnNoSend< <RelayEvent as Message>::Return >
	{
		trace!( "peer: starting Handler<RelayEvent>: {:?}", &re );

		Box::pin( async move
		{
			match re.evt
			{
				// Clean up relays if they disappear
				//
				PeerEvent::Closed | PeerEvent::ClosedByRemote =>
				{
					self.relayed.retain( |_, peer| *peer != re.id );
					self.relays.remove( &re.id );

					let shine = PeerEvent::RelayDisappeared( re.id );
					await!( self.pharos.notify( &shine ));
				}

				_ => {}
			}
		})
	}
}
