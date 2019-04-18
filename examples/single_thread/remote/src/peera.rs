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
		trace!( "Starting peerA" );

		// frame it with multiservice codec
		//
		let (sink_a, stream_a) = await!( listen_tcp( "127.0.0.1:8998" ) );



		// register HandleA with peer as handler for ServiceA
		//
		let mb_handler  : Inbox<HandleA> = Inbox::new()                     ;
		let addr_handler                 = Addr ::new( mb_handler.sender() );

		// Create mailbox for peer
		//
		let mb_peer  : Inbox<MyPeer> = Inbox::new()                  ;
		let peer_addr                = Addr ::new( mb_peer.sender() );

		// create peer with stream/sink + service map
		//
		let mut peer = Peer::new( peer_addr, stream_a.compat(), sink_a.sink_compat() );

		peer.register_service( ServiceA::sid(), box PeerAServices, addr_handler.recipient::<ServiceA>() );
		peer.register_service( ServiceB::sid(), box PeerAServices, addr_handler.recipient::<ServiceB>() );

		let handler = HandleA {};

		mb_peer   .start( peer    ).expect( "Failed to start mailbox of Peer"     );
		mb_handler.start( handler ).expect( "Failed to start mailbox for HandleA" );
	};


	rt::spawn( program ).expect( "Spawn program" );

	rt::run();
}



pub struct HandleA {}

impl Actor for HandleA
{
	fn started ( &mut self ) -> TupleResponse
	{
		async move
		{
			dbg!( "Started HandleA actor" );

		}.boxed()
	}


	fn stopped ( &mut self ) -> TupleResponse
	{
		async move
		{
			dbg!( "Stopped HandleA actor" );

		}.boxed()
	}
}

impl Handler<ServiceA> for HandleA
{
	fn handle( &mut self, msg: ServiceA ) -> Response<ResponseA> { async move
	{
		dbg!( msg );

		ResponseA{ resp: "pong".into() }

	}.boxed() }
}

impl Handler<ServiceB> for HandleA
{
	fn handle( &mut self, msg: ServiceB ) -> Response<()> { async move
	{
		dbg!( msg );

	}.boxed() }
}



