#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, slice_concat_ext ) ]

mod common;
use common::*;

fn main()
{
	flexi_logger::Logger::with_str( "peerb=trace, thespis_impl=trace, tokio=info" ).start().unwrap();

	let program = async move
	{
		trace!( "Starting peerB" );
		let mut peera  = await!( connect_to_tcp( "127.0.0.1:8998" ) );
		let     peera2 = peera.clone();

		// Relay part ---------------------

		let relay = async move
		{
			let (srv_sink, srv_stream) = await!( listen_tcp( "127.0.0.1:8999" ) );

			// Create mailbox for peer
			//
			let mb_peer  : Inbox<MyPeer> = Inbox::new()                  ;
			let peer_addr                = Addr ::new( mb_peer.sender() );

			// create peer with stream/sink
			//
			let mut peer = Peer::new( peer_addr, srv_stream.compat(), srv_sink.sink_compat() );

			peer.register_relayed_service::<ServiceA>( <ServiceA as Service<peer_a::Services>>::sid(), peera2.clone() );
			peer.register_relayed_service::<ServiceB>( <ServiceB as Service<peer_a::Services>>::sid(), peera2         );

			await!( mb_peer.start_fut( peer ) );
		};

		let (relay, relay_outcome) = relay.remote_handle();
		rt::spawn( relay ).expect( "failed to spawn server" );

		// --------------------------------------




		// Call the service and receive the response
		//
		let mut service_a = peer_a::Services::recipient::<ServiceA>( peera.clone() );
		let mut service_b = peer_a::Services::recipient::<ServiceB>( peera.clone() );



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
		await!( peera.send( CloseConnection ) ).expect( "close connection to peera" );

		trace!( "After close connection" );
	};

	rt::spawn( program ).expect( "Spawn program" );

	rt::run();
}
