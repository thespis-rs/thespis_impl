use crate::{ import::*, runtime::rt, remote::*, Addr, Receiver };

/// Type representing Messages coming in over the wire, for internal use only.
//
pub(super) struct Incoming<MS>
{
	pub msg: Result<MS, ThesRemoteErr>
}

impl<MS: 'static + MultiService + Send> Message for Incoming<MS>
{
	type Return = ();
}


/// Handler for incoming messages
//
impl<Out, MS> Handler<Incoming<MS>> for Peer<Out, MS>

	where Out: BoundsOut<MS>,
	      MS : BoundsMS     ,
{
fn handle( &mut self, incoming: Incoming<MS> ) -> ReturnNoSend<()>
{

Box::pin( async move
{
	let cid_null = <MS as MultiService>::ConnID::null();

	let frame = match incoming.msg
	{
		Ok ( mesg  ) => mesg,
		Err( error ) =>
		{
			error!( "Error extracting MultiService from stream: {:#?}", error );

			await!( self.pharos.notify( &PeerEvent::Error( ConnectionError::DeserializationFailure ) ) );

			// Send an error back to the remote peer and close the connection
			//
			await!( self.send_err( cid_null, &ConnectionError::DeserializationFailure, true ) );

			return
		}
	};


	// algorithm for incoming messages. Options are:
	//
	// 1. incoming send/call               for local/relayed/unknown actor (6 options)
	// 2.       response to outgoing call from local/relayed actor         (2 options)
	// 3. error response to outgoing call from local/relayed actor         (2 options)
	//
	// 4 possibilities with ServiceID and ConnID. These can be augmented with
	// predicates about our local state (sid in local table, routing table, unknown), + the codec
	// which gives us largely the 10 needed states:
	//
	// SID  present -> always tells us if it's for local/relayed/unknown actor
	//                 based on our routing tables
	//
	//                 if it's null, it means the message is meant for this peer (ConnectionError).
	//
	// (leaves distinguishing between send/call/response/error)
	//
	// CID   absent  -> Send
	// CID   unknown -> Call
	//
	// CID   present -> Return/Error
	//
	// (leaves Return/Error)
	//
	// sid null      -> ConnectionError
	//
	//
	// TODO: Error handling
	// --------------------
	//
	// A. When calling a remote process, it might send back an error instead of a response
	//
	// B. Incoming messages:
	//
	//    - might fail to deserialize
	//    - might be for an unknown actor
	//    - come in after we are closing down? Verify with the listen function. If there's no
	//      awaits between reception on the stream and here, then we can't see this condition in
	//      both places.
	//


	// I think these should never fail, because they accept random data in the current implementation.
	// However, since it's implementation dependant, and we are generic, we can't know that. It's probably
	// safer to assume that if these do fail we close the connection because all bet's are off for following
	// messages.
	//
	let sid = match frame.service()
	{
		Ok ( sid ) => sid,
		Err( err ) =>
		{
			error!( "Fail to get service_id from incoming frame: {}", err );

			await!( self.pharos.notify( &PeerEvent::Error( ConnectionError::DeserializationFailure ) ) );

			// Send an error back to the remote peer and close the connection
			//
			await!( self.send_err( cid_null, &ConnectionError::DeserializationFailure, true ) );

			return
		}
	};


	let cid = match frame.conn_id()
	{
		Ok ( cid ) => cid,
		Err( err ) =>
		{
			error!( "Fail to get conn_id from incoming frame: {}", err );

			await!( self.pharos.notify( &PeerEvent::Error( ConnectionError::DeserializationFailure ) ) );

			// Send an error back to the remote peer and close the connection
			//
			await!( self.send_err( cid_null, &ConnectionError::DeserializationFailure, true ) );

			return
		}
	};



	// It's a connection error from the remote peer
	//
	// This includes failing to deserialize our messages, failing to relay, unknown service, ...
	//
	if sid.is_null()
	{
		todo!()
	}


	// it's an incoming send
	//
	else if cid.is_null()
	{
		trace!( "Incoming Send" );


		if let Some( (handler, sm) ) = self.services.get( &sid )
		{
			trace!( "Incoming Send for local Actor" );

			sm.send_service( frame, handler );
		}


		// service_id in self.relay => Send to recipient found in self.relay.
		//
		else if let Some( relay_id ) = self.relayed.get( &sid )
		{
			trace!( "Incoming Send for relayed Actor" );

			// We are keeping our internal state consistent, so the unwrap is fine. if it's in
			// self.relayed, it's in self.relays.
			//
			let relay = &mut self.relays.get_mut( &relay_id ).unwrap().0;

			// if this fails, well, the peer is no longer there. Warn remote and observers.
			// We let remote know we are no longer relaying to this service.
			// TODO: should we remove it from self.relayed? Normally there is detection mechanisms
			//       already that should take care of this, but then if they work, we shouldn't be here...
			//       also, if we no longer use unbounded channels, this might fail because the
			//       channel is full.
			//
			if await!( relay.send( frame ) ).is_err()
			{
				let err = ConnectionError::LostRelayBeforeSend( sid.into().to_vec() );
				await!( self.send_err( cid_null, &err, false ) );

				let err = PeerEvent::Error( err );
				await!( self.pharos.notify( &err ) );
			};
		}


		// service_id unknown => send back and log error
		//
		else
		{
			error!( "Incoming Send for unknown Service: {}", &sid );

			// Send an error back to the remote peer and to the observers
			//
			let err = ConnectionError::UnkownService( sid.into().to_vec() );
			await!( self.send_err( cid_null, &err, false ) );

			let err = PeerEvent::Error( err );
			await!( self.pharos.notify( &err ) );
		}
	}


	// it's a response or an error
	//
	else if let Some( channel ) = self.responses.remove( &cid )
	{
		// It's a response
		//
		trace!( "Incoming Return" );

		// TODO: verify our error handling story here. Normally if this
		// fails it means the receiver of the channel was dropped... so
		// they are no longer interested in the reponse? Should we log?
		// Have a different behaviour in release than debug?
		//
		let _ = channel.send( frame );
	}


	// it's a call (!cid.is_null() and cid is unknown)
	//
	else
	{
		trace!( "Incoming Call" );

		if let Some( ref self_addr ) = self.addr
		{
			// It's a call for a local actor
			//
			if let Some( (handler, sm) ) = self.services.get( &sid )
			{
				trace!( "Incoming Call for local Actor" );

				// Call actor
				//
				sm.call_service( frame, handler, self_addr.recipient() );
			}


			// Call for relayed actor
			//
			// service_id in self.relay   => Create Call and send to recipient found in self.relay.
			//
			// What could possibly go wrong???
			// - relay peer has been shut down (eg. remote closed connection)
			// - we manage to call, but then when we await the response, the relay goes down, so the
			//   sender of the channel for the response will come back as a disconnected error.
			//
			else if let Some( peer ) = self.relayed.get( &sid )
			{
				trace!( "Incoming Call for relayed Actor" );

				// The unwrap is safe, because we just checked self.relayed and we shall keep both
				// in sync
				//
				let mut peer      = self.relays.get_mut( &peer ).unwrap().0.clone();
				let mut self_addr = self_addr.clone();

				rt::spawn( async move
				{
					// until we have bounded channels, this should never fail, so I'm leaving the expect.
					//
					let channel = await!( peer.call( Call::new( frame ) ) ).expect( "Call to relay failed" );


					// TODO: Let the incoming remote know that their call failed
					// not sure the current process wants to know much about this. We sure shouldn't
					// crash.
					//
					match channel
					{
						Ok( receiver ) =>	{match await!( receiver )
						{

							Ok( resp ) =>
							{
								trace!( "Got response from relayed call, sending out" );

								// until we have bounded channels, this should never fail, so I'm leaving the expect.
								//
								await!( self_addr.send( resp ) ).expect( "Failed to send response from relay out on connection" );
							},


							// This can only happen if the sender got dropped. Eg, if the remote relay goes down
							// Inform peer that their call failed because we lost connection to the relay after
							// it was sent out.
							//
							Err( _err ) =>
							{
								// Send an error back to the remote peer
								//
								let err = Self::prep_error( cid, &ConnectionError::LostRelayBeforeResponse );

								await!( self_addr.send( err ) ).expect( "send msg to self" );
							}
						}},

						// Sending out call by relay failed
						//
						Err( _err ) =>
						{
							// Send an error back to the remote peer
							//
							let err = Self::prep_error( cid, &ConnectionError::LostRelayBeforeCall );

							await!( self_addr.send( err ) ).expect( "send msg to self" );
						}
					}

				}).expect( "failed to spawn" );

			}

			// service_id unknown => send back and log error
			//
			else
			{
				error!( "Incoming Call for unknown Actor: {}", sid );

				// Send an error back to the remote peer and to the observers
				//
				let err = ConnectionError::UnkownService( sid.into().to_vec() );
				await!( self.send_err( cid_null, &err, false ) );

				let err = PeerEvent::Error( err );
				await!( self.pharos.notify( &err ) );
			}
		}

		else
		{
			// we no longer have our address, we're shutting down. we can't really do anything
			// without our address we won't have the sink for the connection either. We can
			// no longer send outgoing messages
			//
			return
		}
	}

}) // End of Box::pin( async move

} // end of handle
} // end of impl Handler
