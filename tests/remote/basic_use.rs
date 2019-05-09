use crate::*;

// Test basic remote funcionality. Test intertwined sends and calls.
//
#[test]
//
fn remote()
{
	let peera = async
	{
		// get a framed connection
		//
		let (sink_a, stream_a) = await!( listen_tcp( "127.0.0.1:8998" ) );


		// Create mailbox for our handler
		//
		let addr_handler = Addr::try_from( Sum(0) ).expect( "spawn actor mailbox" );

		// Create mailbox for peer
		//
		let mb_peer  : Inbox<MyPeer> = Inbox::new()                  ;
		let peer_addr                = Addr ::new( mb_peer.sender() );

		// create peer with stream/sink
		//
		let mut peer = Peer::new( peer_addr, stream_a.compat(), sink_a.sink_compat() ).expect( "spawn peer" );

		// register Sum with peer as handler for Add and Show
		//
		peer.register_service::<Add , remotes::Services>( addr_handler.recipient() );
		peer.register_service::<Show, remotes::Services>( addr_handler.recipient() );

		mb_peer.start( peer ).expect( "Failed to start mailbox of Peer" );
	};


	let peerb = async
	{
		let (mut peera, _)  = await!( connect_to_tcp( "127.0.0.1:8998" ) );

		// Call the service and receive the response
		//
		let mut add  = remotes::Services::recipient::<Add >( peera.clone() );
		let mut show = remotes::Services::recipient::<Show>( peera.clone() );

		let resp = await!( add.call( Add(5) ) ).expect( "Call failed" );
		assert_eq!( (), resp );

		await!( add.send( Add(5) ) ).expect( "Send failed" );

		let resp = await!( show.call( Show ) ).expect( "Call failed" );
		assert_eq!( 10, resp );

		await!( peera.send( CloseConnection{ remote: false } ) ).expect( "close connection to peera" );
	};


	// As far as I can tell, execution order is not defined, so hmm, there is no
	// guarantee that a is listening before b tries to connect, but it seems to work for now.
	//
	rt::spawn( peera  ).expect( "Spawn peera"  );
	rt::spawn( peerb  ).expect( "Spawn peerb"  );

	rt::run();
}





#[ derive( Actor ) ]
//
pub struct Parallel
{
	pub sum: Box< Recipient<Show> >,
}


impl Handler< Show > for Parallel
{
	fn handle( &mut self, _: Show ) -> ReturnNoSend<u64> { Box::pin( async move
	{
		await!( self.sum.call( Show ) ).expect( "call sum" )

	})}
}


service_map!
(
	namespace     : parallel ;
	peer_type     : MyPeer   ;
	multi_service : MS       ;
	services      : Show     ;
);




// Test correct async behavior. Verify that a peer can continue to
// send/receive while waiting for the response to a call.
//
#[test]
//
fn parallel()
{
	let peera = async
	{
		// get a framed connection
		//
		let (sink_a, stream_a) = await!( listen_tcp( "127.0.0.1:20001" ) );

		// Create mailbox for peer
		//
		let mb_peer  : Inbox<MyPeer> = Inbox::new()                  ;
		let peer_addr                = Addr ::new( mb_peer.sender() );

		// create peer with stream/sink
		//
		let mut peer = Peer::new( peer_addr.clone(), stream_a.compat(), sink_a.sink_compat() ).expect( "spawn peer" );

		// Create recipients
		//
		let show = remotes::Services::recipient::<Show>( peer_addr.clone() );

		// Create mailbox for our handler
		//
		let addr_handler = Addr::try_from( Parallel{ sum: box show } ).expect( "spawn actor mailbox" );

		// register Sum with peer as handler for Add and Show
		//
		peer.register_service::<Show, parallel::Services>( addr_handler.recipient() );

		mb_peer.start( peer ).expect( "Failed to start mailbox of Peer" );
	};


	let peerb = async
	{
		let (sink_b, stream_b) = await!( connect_return_stream( "127.0.0.1:20001" ) );

		// Create mailbox for peer
		//
		let     mb_peer  : Inbox<MyPeer> = Inbox::new()                  ;
		let mut peer_addr                = Addr ::new( mb_peer.sender() );

		// create peer with stream/sink
		//
		let mut peer = Peer::new( peer_addr.clone(), stream_b.compat(), sink_b.sink_compat() ).expect( "spawn peer" );

		// Create mailbox for our handler
		//
		let addr_handler = Addr::try_from( Sum(19) ).expect( "spawn actor mailbox" );


		// register Sum with peer as handler for Add and Show
		//
		peer.register_service::<Show, remotes::Services>( addr_handler.recipient() );

		mb_peer.start( peer ).expect( "Failed to start mailbox of Peer" );


		// Create recipients
		//
		let mut show = parallel::Services::recipient::<Show>( peer_addr.clone() );

		let resp = await!( show.call( Show ) ).expect( "Call failed" );
		assert_eq!( 19, resp );

		// dbg!( resp );

		await!( peer_addr.send( CloseConnection{ remote: false } ) ).expect( "close connection to peera" );
	};


	rt::spawn( peera  ).expect( "Spawn peera"  );
	rt::spawn( peerb  ).expect( "Spawn peerb"  );

	rt::run();
}




// Test basic remote funcionality. Test intertwined sends and calls.
//
#[test]
//
fn call_after_close_connection()
{
	let nodea = async
	{
		// get a framed connection
		//
		let _ = await!( listen_tcp( "127.0.0.1:20002" ) );
	};


	let nodeb = async
	{
		let (peera, mut peera_evts)  = await!( connect_to_tcp( "127.0.0.1:20002" ) );

		// Call the service and receive the response
		//
		let mut add = remotes::Services::recipient::<Add>( peera.clone() );

		assert_eq!( PeerEvent::ClosedByRemote, await!( peera_evts.next() ).unwrap() );

		match await!( add.call( Add(5) ) )
		{
			Ok (_) => unreachable!(),
			Err(e) =>
			{
				match e.downcast::<ThesError>()
				{
					Ok (e    ) => assert_eq!( ThesError::PeerSendAfterCloseConnection, e ),
					Err(wrong) => panic!( wrong ),
				}
			}
		}
	};


	// As far as I can tell, execution order is not defined, so hmm, there is no
	// guarantee that a is listening before b tries to connect, but it seems to work for now.
	//
	rt::spawn( nodea  ).expect( "Spawn peera"  );
	rt::spawn( nodeb  ).expect( "Spawn peerb"  );

	rt::run();
}

