use { crate :: { import::*, remote::peer::* }};

/// Control message for [Peer]. The peer needs it's own address for normal operation,
/// so normally it will never drop, even if you drop all addresses you have of it.
/// Since it will never drop, it's mailbox will never stop listening for incoming messages
/// and the task will never complete, preventing your program from shutting down naturally.
///
/// With this message you can tell the peer to drop it's copy of it's own address. You still
/// have to drop your copies... otherwise the peer won't be dropped, but the peer will no
/// longer accept incoming Calls. Sends will still be processed, because once they have
/// arrived, the connection is no longer needed for them to be processed.
///
/// On an incoming call, an error shall be sent back to the other process.
///
/// The peer will also drop it's outgoing Sink, so the other end of the connection
/// will be notified that we close it.
///
/// If the remote closes the connection, all of this will happen automatically.
//
pub struct CloseConnection
{
	/// informs the peer whether the connection was closed remotely. If you close
	/// manually, set to false. The main effect of this is that the peer will send
	/// PeerEvents::ConnectionClosedByRemote to observers instead of PeerEvent::ConnectionClosed.
	//
	pub remote: bool,
}

impl Message for CloseConnection { type Return = (); }

impl<Out, MS> Handler<CloseConnection> for Peer<Out, MS>

	where Out: BoundsOut<MS> ,
	      MS : BoundsMS      ,

{
	fn handle( &mut self, msg: CloseConnection ) -> ReturnNoSend<()>
	{
		Box::pin( async move
		{
			trace!( "CloseConnection self in peer");


			match msg.remote
			{
				true  => await!( self.pharos.notify( &PeerEvent::ClosedByRemote ) ),
				false => await!( self.pharos.notify( &PeerEvent::Closed         ) ),
			}


			// Try to close the connection properly
			//
			match &mut self.outgoing
			{
				Some( out ) =>
				{
					await!( out.close() ).expect( "close sink for peer" );
					self.outgoing = None;
				},

				None => {},
			};


			// try to drop close our mailbox and drop ourselves
			//
			self.addr          = None;
			self.listen_handle = None;


			// Also clear everything else, because services might have our address, because they
			// want to send stuff over the network, so if we keep them alive, they will keep us
			// alive. This breaks that cycle.
			//
			self.services .clear();
			self.relays   .clear();
			self.responses.clear();
		})
	}
}
