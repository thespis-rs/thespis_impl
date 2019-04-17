#![ allow( unused_imports, dead_code ) ]
#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait ) ]

mod common;

use common::*;


fn main()
{
	rt::init( box TokioRT::default() ).expect( "We only set the executor once" );
	simple_logger::init().unwrap();

	let program = async move
	{
		trace!( "Starting peerA" );

		// create tcp server
		//
		let socket   = "127.0.0.1:8998".parse::<SocketAddr>().unwrap();
		let listener = TcpListener::bind( &socket ).expect( "bind address" );

		let codec: MulServTokioCodec<MS> = MulServTokioCodec::new();

		let stream   = await01!( listener.incoming().take(1).into_future() )
			.expect( "find one stream" ).0
			.expect( "find one stream" )
		;


		// frame it with multiservice codec
		//
		let (sink_a, stream_a) = codec.framed( stream ).split();

		// Create mailbox for peer
		//
		let mb  : Inbox<MyPeer> = Inbox::new()             ;
		let addr                = Addr ::new( mb.sender() );

		// create peer with stream/sink + service map
		//
		let mut peer = Peer::new( addr, stream_a.compat(), sink_a.sink_compat() );

		// register HandleA with peer as handler for ServiceA
		//
		let mbh       : Inbox<HandleA>  = Inbox::new()              ;
		let addrh                       = Addr ::new( mbh.sender() );
		let handler                     = HandleA {}                ;
		let recipienth                  = addrh.recipient::<ServiceA>();
		let rech      : Rcpnt<ServiceA> = Rcpnt::new( recipienth );

		peer.register_service( ServiceA::sid(), box PeerAServices, box rech );

		mb .start( peer    ).expect( "Failed to start mailbox of Peer"     );
		mbh.start( handler ).expect( "Failed to start mailbox for HandleA" );
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
