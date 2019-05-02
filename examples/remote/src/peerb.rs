#![ feature( await_macro, async_await, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, slice_concat_ext ) ]

mod common;
use common::*;

fn main()
{
	flexi_logger::Logger::with_str( "peerb=trace, thespis_impl=trace, tokio=info" ).start().unwrap();

	let program = async move
	{
		trace!( "Starting peerB" );

		let (srv_sink, srv_stream) = await!( connect_return_stream( "127.0.0.1:8998" ) );


		// Create mailbox for peer
		//
		let     mb_peera  : Inbox<MyPeer> = Inbox::new()                   ;
		let mut peera_addr                = Addr ::new( mb_peera.sender() );
		let     peera_addr2               = peera_addr.clone()             ;

		// create peer with stream/sink + service map
		//
		let mut peeraa = Peer::new( peera_addr.clone(), srv_stream.compat(), srv_sink.sink_compat() ).expect( "spawn peer" );

		let peera_evts = peeraa.observe( 10 );

		mb_peera.start( peeraa ).expect( "spawn peer for peera" );

		// Relay part ---------------------

		let relay = async move
		{
			let (srv_sink, srv_stream) = await!( listen_tcp( "127.0.0.1:8999" ) );

			// Create mailbox for peer
			//
			let mb_peer  : Inbox<MyPeer> = Inbox::new()                  ;
			let mut peer_addr            = Addr ::new( mb_peer.sender() );
			// create peer with stream/sink
			//
			let peer = Peer::new( peer_addr.clone(), srv_stream.compat(), srv_sink.sink_compat() ).expect( "spawn peer" );

			let registration = async move
			{
				let add  = <ServiceA as Service<peer_a::Services>>::sid();
				let show = <ServiceB as Service<peer_a::Services>>::sid();
				let regi = RegisterRelay{ services: vec![ add, show ], peer_events: peera_evts, peer: peera_addr2 };

				await!( peer_addr.call( regi ) ).expect( "Send register_relay" ).expect( "register relayed services" );
			};

			rt::spawn( registration ).expect( "spawn registration" );

			await!( mb_peer.start_fut( peer ) );
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
