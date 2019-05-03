#![ feature( await_macro, async_await, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, slice_concat_ext ) ]

mod common;
use common::*;

fn main()
{
	flexi_logger::Logger::with_str( "peerb=trace, thespis_impl=trace, tokio=info" ).start().unwrap();

	let program = async move
	{
		trace!( "Starting peerB" );

		let (mut peera_addr, peera_evts) = await!( connect_to_tcp( "127.0.0.1:8998" ) );
		let peera_addr2 = peera_addr.clone();

		// Relay part ---------------------

		let relay = async move
		{
			let (srv_sink, srv_stream) = await!( listen_tcp( "127.0.0.1:8999" ) );

			// Create mailbox for peer
			//
			let     mb_peerc  : Inbox<MyPeer> = Inbox::new()                  ;
			let mut peer_addr                 = Addr ::new( mb_peerc.sender() );

			// create peer with stream/sink
			//
			let mut peerc = Peer::new( peer_addr.clone(), srv_stream.compat(), srv_sink.sink_compat() )

				.expect( "spawn peerc" )
			;

			// Get the event stream
			//
			let peerc_evts = peerc.observe( 10 );

			// Start the mailbox for peerc
			//
			mb_peerc.start( peerc ).expect( "spawn peerc mb" );

			// Register the services to be relayed
			//
			let add  = <ServiceA as Service<peer_a::Services>>::sid();
			let show = <ServiceB as Service<peer_a::Services>>::sid();
			let regi = RegisterRelay{ services: vec![ add, show ], peer_events: peera_evts, peer: peera_addr2 };

			await!( peer_addr.call( regi ) ).expect( "Send register_relay" ).expect( "register relayed services" );

			// Wait until the connection closes. If you need more fine grained info from the peer,
			// you can inspect the elements of this stream, but this is an easy trick to just detect
			// the dropping of the peer.
			//
			await!( peerc_evts.into_future() );
		};

		let (relay, relay_outcome) = relay.remote_handle();
		rt::spawn( relay ).expect( "failed to spawn server" );

		// --------------------------------------




		// Call the service and receive the response
		//
		let mut service_a = peer_a::Services::recipient::<ServiceA>( peera_addr.clone() );
		let mut service_b = peer_a::Services::recipient::<ServiceB>( peera_addr.clone() );



		let resp = await!( service_a.call( ServiceA{ msg: "hi from peerb".to_string() } ) ).expect( "Call failed" );

		dbg!( resp );



		// Send
		//
		await!( service_b.send( ServiceB{ msg: "SEND from peerb".to_string() } ) ).expect( "Send failed" );



		let resp = await!( service_a.call( ServiceA{ msg: "hi from peerb -- again!!!".to_string() } ) ).expect( "Call failed" );

		dbg!( resp );


		// Send
		//
		await!( service_b.send( ServiceB{ msg: "SEND AGAIN from peerb".to_string() } ) ).expect( "Send failed" );


		// Call ServiceB
		let resp = await!( service_b.call( ServiceB{ msg: "hi from peerb -- Calling to ServiceB!!!".to_string() } ) )

			.expect( "Call failed" );

		dbg!( resp );

		// If the peerc closes the connection, we might as well terminate as well, because we did all our work.
		//
		await!( relay_outcome );
		await!( peera_addr.send( CloseConnection{ remote: false } ) ).expect( "close connection to peera" );

		trace!( "After close connection" );
	};

	rt::spawn( program ).expect( "Spawn program" );

	rt::run();
}
