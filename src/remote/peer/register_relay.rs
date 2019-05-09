use crate :: { import::*, runtime::rt, remote::{ *, peer::peer_event::RelayEvent }, Addr };


/// Type representing the outgoing call. Used by a recipient to a remote service to communicate
/// an outgoing call to [Peer]. Also used by [Peer] to call a remote service when relaying.
///
/// MS must be of the same type as the type parameter on [Peer].
//
pub struct RegisterRelay<Out, MS>

	where Out: BoundsOut<MS>,
	      MS : BoundsMS     ,

{
	pub services   : Vec<&'static <MS as MultiService>::ServiceID> ,
	pub peer       : Addr<Peer<Out, MS>>                           ,
	pub peer_events: mpsc::Receiver<PeerEvent>                     ,
}

impl<MS, Out> Message for RegisterRelay<Out, MS>

	where Out: BoundsOut<MS>,
	      MS : BoundsMS     ,

{
	type Return = Result<(), ThesRemoteErr>;
}



/// Handler for RegisterRelay
///
/// Tell this peer to make a given service avaible to a remote, by forwarding incoming requests to the given peer.
/// For relaying services from other processes. You should normally use the method [`register_relayed_services`]
/// rather than sending this message, so that your peer is completely set up before starting it's mailbox. However
/// it can happen that the connection to the relay is lost, but you want to reconnect at runtime and resume relaying,
/// so this is provided for that scenario, since you won't be able to call [`register_relayed_services`] once your
/// peer has been started.
///
/// TODO: - verify we can relay services unknown at compile time. Eg. could a remote process ask in runtime
///       could you please relay for me. We just removed a type parameter here, which should help, but we
///       need to test it to make sure it works.
///
// Design:
// - take a peer with a vec of services to relay over that peer.
// - store in a hashmap, but put the peer address in an Rc? + a unique id (addr doesn't have Eq)
//
//
impl<Out, MS> Handler< RegisterRelay<Out, MS> > for Peer<Out, MS>

	where Out: BoundsOut<MS>,
	      MS : BoundsMS     ,
{
	fn handle( &mut self, msg: RegisterRelay<Out, MS> ) -> ReturnNoSend< <RegisterRelay<Out, MS> as Message>::Return >
	{
		trace!( "peer: starting Handler<RegisterRelay<Out, MS>>" );

		let RegisterRelay { services, peer, peer_events } = msg;

		Box::pin( async move
		{
			self.register_relayed_services( services, peer, peer_events )
		})
	}
}
