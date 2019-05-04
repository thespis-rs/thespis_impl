use crate::{ import::*, remote::* };


/// Events that can happen during the lifecycle of the peer. Use the [`observe`] method to subscribe to events.
///
/// When you see either `Closed` or `ClosedByRemote`, the connection is lost and you should drop all
/// addresses/recipients you hold for this peer, so it can be dropped. You can no longer send messages
/// over this peer after these events.
//
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



/// Handler for events from relays.
/// If we notice Closed or ClosedByRemote on relays, we will stop relaying their services.
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
