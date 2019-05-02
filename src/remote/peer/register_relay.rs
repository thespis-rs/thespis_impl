use crate :: { import::*, ThesError, runtime::rt, remote::{ *, peer::peer_event::RelayEvent }, Addr };


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
	type Return = ThesRes<()>;
}



/// Handler for RegisterRelay
///
/// Tell this peer to make a given service avaible to a remote, by forwarding incoming requests to the given peer.
/// For relaying services from other processes.
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
			let mut addr = match &self.addr
			{
				Some( addr ) => addr.clone() ,
				None         => return Ok(()),
			};

			let peer_id = < Addr<Self> as Recipient<RelayEvent> >::actor_id( &peer );


			let listen = async move
			{
				// We need to map this to a custom type, since we had to impl Message for it.
				//
				let stream = &mut peer_events.map( |evt| RelayEvent{ id: peer_id, evt } );

				// This can fail if:
				// - channel is full (for now we use unbounded)
				// - the receiver is dropped. The receiver is our mailbox, so it should never be dropped
				//   as long as we have an address to it.
				//
				// So, I think we can unwrap for now.
				//
				await!( addr.send_all( stream ) ).expect( "peer send to self");

				// Same as above.
				// Normally relays shouldn't just dissappear, without notifying us, but it could
				// happen for example that the peer already shut down and the above stream was already
				// finished, we would immediately be here, so we do need to clean up.
				// Since we are doing multi threading it's possible to receive the peers address,
				// but it's no longer valid. So send ourselves a message.
				//
				let evt = PeerEvent::Closed;

				await!( addr.send( RelayEvent{ id: peer_id, evt } ) ).expect( "peer send to self");
			};

			// When we need to stop listening, we have to drop this future, because it contains
			// our address, and we won't be dropped as long as there are adresses around.
			//
			let (remote, handle) = listen.remote_handle();
			rt::spawn( remote )?;

			// self.relayed       : HashMap< &'static <MS as MultiService>::ServiceID, usize >,
			// self.relays        : HashMap< usize, (Addr<Self>, RemoteHandle<()>)           >,

			self.relays .insert( peer_id, (peer, handle) );

			for sid in services
			{
				trace!( "Register relaying to: {}", sid );
				self.relayed.insert( sid, peer_id );
			}

			Ok(())
		})
	}
}
