#![ feature( await_macro, async_await, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, slice_concat_ext ) ]

mod common;

use common::*;


fn main()
{
	// flexi_logger::Logger::with_str( "peera=trace, thespis_impl=trace, tokio=warn" ).start().unwrap();

	let program = async move
	{
		trace!( "Starting peerA" );

		// register HandleA with peer as handler for ServiceA
		//
		let addr_handler = Addr::try_from( HandleA ).expect( "spawn mb HandleA" );

		// Create peera network connection actor
		//
		let (sink_a, stream_a)       = await!( listen_tcp( "127.0.0.1:8998" ) );
		let mb_peer  : Inbox<MyPeer> = Inbox::new()                  ;
		let peer_addr                = Addr ::new( mb_peer.sender() );

		// create peer with stream/sink + service map
		//
		let mut peer = Peer::new( peer_addr, stream_a.compat(), sink_a.sink_compat() ).expect( "create peer" );

		// Register our services
		//
		peer.register_service::<ServiceA, peer_a::Services>( addr_handler.recipient() );
		peer.register_service::<ServiceB, peer_a::Services>( addr_handler.recipient() );

		mb_peer.start( peer ).expect( "Failed to start mailbox of Peer" );
	};


	rt::spawn( program ).expect( "Spawn program" );

	rt::run();
}



pub struct HandleA;

impl Actor for HandleA
{
	fn started ( &mut self ) -> Return<()>
	{
		Box::pin( async move
		{
			trace!( "Started HandleA actor" );
		})
	}


	fn stopped ( &mut self ) -> Return<()>
	{
		Box::pin( async move
		{
			trace!( "Stopped HandleA actor" );
		})
	}
}


impl Handler<ServiceA> for HandleA
{
	fn handle( &mut self, msg: ServiceA ) -> ReturnNoSend<ReturnA> { Box::pin( async move
	{
		dbg!( msg );

		ReturnA{ resp: "pong".into() }

	})}
}


impl Handler<ServiceB> for HandleA
{
	fn handle( &mut self, msg: ServiceB ) -> ReturnNoSend<()> { Box::pin( async move
	{
		dbg!( msg );

	})}
}



