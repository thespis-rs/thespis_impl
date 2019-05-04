use super::*;

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
		peer.register_service::<Add , remote::Services>( addr_handler.recipient() );
		peer.register_service::<Show, remote::Services>( addr_handler.recipient() );

		mb_peer.start( peer ).expect( "Failed to start mailbox of Peer" );
	};


	let peerb = async
	{
		let mut peera  = await!( connect_to_tcp( "127.0.0.1:8998" ) );

		// Call the service and receive the response
		//
		let mut add  = remote::Services::recipient::<Add >( peera.clone() );
		let mut show = remote::Services::recipient::<Show>( peera.clone() );

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
