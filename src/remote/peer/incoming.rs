use crate::{ import::*, ThesError, runtime::rt, remote::*, Addr, Receiver };

/// Type representing Messages coming in over the wire, for internal use only.
//
pub(super) struct Incoming<MS>
{
	pub msg: ThesRes<MS>
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
fn handle( &mut self, incoming: Incoming<MS> ) -> Return<()>
{
trace!( "got incoming event on stream" );

async move
{
	let frame = match incoming.msg
	{
		Ok ( mesg  ) => mesg,

		Err( error ) =>
		{
			error!( "Error extracting MultiService from stream: {:#?}", error );

			await!( self.pharos.notify( &PeerEvent::Error( ConnectionError::DeserializationFailure ) ) );

			// TODO: we should send an error to the remote before closing the connection.
			//
			if let Some( addr ) = &mut self.addr
			{
				// until we have bounded channels, this should never fail, so I'm leaving the expect.
				//
				await!( addr.send( CloseConnection{ remote: false } ) ).expect( "Send Drop to self in Peer" );
			}

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
			// TODO: Send error back to remote saying that deserialization failed
			//
			error!( "Fail to get service_id from incoming frame: {}", err );

			await!( self.pharos.notify( &PeerEvent::Error( ConnectionError::DeserializationFailure ) ) );


			if let Some( ref mut addr ) = self.addr
			{
				// until we have bounded channels, this should never fail, so I'm leaving the expect.
				//
				await!( addr.send( CloseConnection{ remote: false } ) ).expect( "send close connection");
			}

			return
		}
	};

	let cid = match frame.conn_id()
	{
		Ok ( cid ) => cid,
		Err( err ) =>
		{
			// TODO: Send error back to remote saying that deserialization failed
			//
			error!( "Fail to get conn_id from incoming frame: {}", err );

			await!( self.pharos.notify( &PeerEvent::Error( ConnectionError::DeserializationFailure ) ) );


			if let Some( ref mut addr ) = self.addr
			{
				// until we have bounded channels, this should never fail, so I'm leaving the expect.
				//
				await!( addr.send( CloseConnection{ remote: false } ) ).expect( "send close connection");
			}

			return
		}
	};


	// it's an incoming send
	//
	if cid.is_null()
	{
		trace!( "Incoming Send" );


		if let Some( (handler, sm) ) = self.services.get( &sid )
		{
			trace!( "Incoming Send for local Actor" );

			sm.send_service( frame, handler );
		}


		// service_id in self.relay => Create Call and send to recipient found in self.relay.
		//
		else if let Some( relay ) = self.relay.get_mut( &sid )
		{
			trace!( "Incoming Send for relayed Actor" );

			// until we have bounded channels, this should never fail, so I'm leaving the expect.
			//
			await!( relay.send( frame ) ).expect( "relaying send to other peer" );
		}

		// service_id unknown => send back and log error
		// TODO: send error back
		//
		else
		{
			error!( "Incoming Call for unknown Service: {}", &sid );
		}
	}


	// it's a response or an error
	//
	else if let Some( channel ) = self.responses.remove( &cid )
	{
		// It's a response
		//
		if !sid.is_null()
		{
			trace!( "Incoming Return" );

			// TODO: verify our error handling story here. Normally if this
			// fails it means the receiver of the channel was dropped... so
			// they are no longer interested in the reponse? Should we log?
			// Have a different behaviour in release than debug?
			//
			let _ = channel.send( frame );
		}

		// TODO: it's an error deserialize it and send it to observers
		//
		else
		{
			trace!( "Incoming Error" );
		}
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
			else if let Some( peer ) = self.relay.get_mut( &sid )
			{
				trace!( "Incoming Call for relayed Actor" );

				let mut peer      = peer     .clone();
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
						Ok( receiver ) =>	{	match await!( receiver )
						{
							Ok( resp ) =>
							{
								trace!( "Got response from relayed call, sending out" );

								// until we have bounded channels, this should never fail, so I'm leaving the expect.
								//
								await!( self_addr.send( resp ) ).expect( "Failed to send response from relay out on connection" );
							},

							// TODO: This can only happen if the sender got dropped. Eg, if the remote relay goes down
							// Inform peer that their call failed because we lost connection to the relay after
							// it was sent out.
							//
							Err( _err ) => {}
						}},

						// Sending out call by relay failed
						// TODO: inform remote
						//
						Err( _err ) => {}
					}


				}).expect( "failed to spawn" );

			}

			// service_id unknown => send back and log error
			//
			else
			{
				trace!( "Incoming Call for unknown Actor" );
			}
		}

		else
		{
			// we no longer have our address, we're shutting down.
			// TODO: We should let the caller know by sending an error back.
			//
			// don't process frame any more
			//
			return
		}
	}

}.boxed()
}
}
