use { crate :: { import::*, ThesError, runtime::rt, remote::*, single_thread::{ Addr, Rcpnt } } };


/// Type representing Messages coming in over the wire, for internal use only.
//
pub(super) struct Incoming<MulService: MultiService>
{
	pub mesg: MulService,
}

impl<MulService: 'static + MultiService> Message for Incoming<MulService>
{
	type Return = ();
}


/// Handler for incoming messages
//
impl<Out, MulService> Handler<Incoming<MulService>> for Peer<Out, MulService>

	where Out       : BoundsOut<MulService>,
	      MulService: BoundsMulService     ,
{
	fn handle( &mut self, incoming: Incoming<MulService> ) -> Return<()>
	{
		trace!( "Peer: Incoming message!" );

		async move
		{
			let frame = incoming.mesg;

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
			// However, since it's implementation dependant, in we are generic, we can't know that. It's probably
			// safer to assume that if these do fail we close the connection because all bet's are off for following
			// messages.
			//
			let sid = match frame.service()
			{
				Ok ( sid ) => sid,
				Err( err ) =>
				{
					// TODO: Send error back to remote saying that deserialization failed
					// Close connection because corrupt?
					// don't try to process this frame any further
					//
					debug!( "Fail to get service_id from incoming frame: {}", err );

					if let Some( ref mut addr ) = self.addr
					{
						await!( addr.send( CloseConnection ) ).expect( "send close connection");
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
					// Close connection because corrupt?
					// don't try to process this frame any further
					//
					debug!( "Fail to get conn_id from incoming frame: {}", err );

					if let Some( ref mut addr ) = self.addr
					{
						await!( addr.send( CloseConnection ) ).expect( "send close connection");
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


				// service_id in self.relay   => Create Call and send to recipient found in self.relay.
				//
				else if let Some( relay ) = self.relay.get_mut( &sid )
				{
					trace!( "Incoming Send for relayed Actor" );

					await!( relay.send( frame ) ).expect( "relaying send to other peer" );
				}

				// service_id unknown => send back and log error
				// TODO: send error back
				//
				else
				{
					error!( "Incoming Call for unknown Actor" );
				}
			}


			// it's a response or an error
			//
			else if let Some( channel ) = self.responses.remove( &cid )
			{
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

				// TODO: it's an error
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
						sm.call_service( frame, handler, self_addr.recipient::<MulService>() );
					}


					// service_id in self.relay   => Create Call and send to recipient found in self.relay.
					//
					else if let Some( peer ) = self.relay.get_mut( &sid )
					{
						trace!( "Incoming Call for relayed Actor" );

						let mut peer      = peer     .clone();
						let mut self_addr = self_addr.clone();

						rt::spawn( async move
						{
							let channel = await!( peer.call( Call::new( frame ) ) ).expect( "Call to relay failed" );

							let resp    = await!( channel.expect( "send call out over connection" ) )

								.expect( "failed to read from channel for response from relay" );

							trace!( "Got response from relayed call, sending out" );

							await!( self_addr.send( resp ) ).expect( "Failed to send response from relay out on connection" );

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
