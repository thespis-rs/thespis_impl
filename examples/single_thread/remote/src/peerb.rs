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
		trace!( "Starting peerB" );

		// Connect to tcp server
		//
		let socket = "127.0.0.1:8998".parse::<SocketAddr>().unwrap();
		let stream = await01!( TcpStream::connect( &socket ) ).expect( "connect address" );

		// frame the connection with codec for multiservice
		//
		let codec: MulServTokioCodec<MS> = MulServTokioCodec::new();

		let (sink_a, stream_a) = codec.framed( stream ).split();

		// Create mailbox for peer
		//
		let mb  : Inbox<MyPeer> = Inbox::new()             ;
		let addr                = Addr ::new( mb.sender() );

		// create peer with stream/sink + service map
		//
		let peer = Peer::new( addr.clone(), stream_a.compat(), sink_a.sink_compat() );

		mb.start( peer ).expect( "Failed to start mailbox" );

		// Call the service and receive the response
		//
		let mut recipient = PeerAServices::recip_service_a( addr );

		let resp = await!( recipient.call( ServiceA{ msg: "hi from peerb".to_string() } ) )

			.expect( "Call failed" )
		;

		dbg!( resp );
	};

	rt::spawn( program ).expect( "Spawn program" );

	rt::run();
}
