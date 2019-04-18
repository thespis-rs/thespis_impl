#![ allow( unused_imports, dead_code ) ]
#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, slice_concat_ext ) ]

mod common;
use common::*;

fn main()
{
	rt::init( box TokioRT::default() ).expect( "We only set the executor once" );
	simple_logger::init_with_level( log::Level::Debug ).unwrap();

	let program = async move
	{
		trace!( "Starting peerB" );
		let peera  = await!( connect_to_tcp( "127.0.0.1:8998" ) );
		let peera2 = peera.clone();

		// Relay part ---------------------

		rt::spawn( async move
		{
			let (srv_sink, srv_stream) = await!( listen_tcp( "127.0.0.1:8999" ) );

			// Create mailbox for peer
			//
			let mb_peer  : Inbox<MyPeer> = Inbox::new()                  ;
			let peer_addr                = Addr ::new( mb_peer.sender() );

			// create peer with stream/sink + service map
			//
			let mut peer = Peer::new( peer_addr, srv_stream.compat(), srv_sink.sink_compat() );

			peer.register_relayed_service::<ServiceA>( ServiceA::sid(), peera2.clone() );
			peer.register_relayed_service::<ServiceB>( ServiceB::sid(), peera2         );

			mb_peer.start( peer ).expect( "Failed to start mailbox of Peer" );

		}).expect( "failed to spawn server" );

		// --------------------------------------




		// Call the service and receive the response
		//
		let mut service_a = PeerAServices::recip_service_a( peera.clone() );
		let mut service_b = PeerAServices::recip_service_b( peera         );



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
	};

	rt::spawn( program ).expect( "Spawn program" );

	rt::run();
}
