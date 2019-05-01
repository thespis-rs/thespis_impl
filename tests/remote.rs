#![ cfg( feature = "remote" ) ]

#![ feature( await_macro, async_await, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, optin_builtin_traits ) ]

// What is tested:
//
// 1. basic remote functionality
// 2. relays
// 3. verify we work in paralell. A calls B, B calls A back for some extra info before answering the first call.
// 4. use the same connection for services on both ends -> probably combine this with 3.




use
{
	thespis      :: { *                                          } ,
	thespis_impl :: { *, remote::*, service_map, runtime::{ rt } } ,
	std          :: { net::SocketAddr                            } ,


	futures      ::
	{
		compat :: { Compat01As03Sink, Stream01CompatExt, Sink01CompatExt, Future01CompatExt } ,
	},


	tokio        ::
	{
		prelude :: { Stream as TokStream, stream::{ SplitStream as TokSplitStream, SplitSink as TokSplitSink } } ,
 		net     :: { TcpStream, TcpListener                                                                    } ,
		codec   :: { Decoder, Framed                                                                           } ,
	},
};


mod common;

use common::import::*;
use common::actors::*;


pub type TheSink = Compat01As03Sink<TokSplitSink<Framed<TcpStream, MulServTokioCodec<MS>>>, MS> ;
pub type MS      = MultiServiceImpl<ServiceID, ConnID, Codecs>                                  ;
pub type MyPeer  = Peer<TheSink, MS>                                                            ;



pub async fn listen_tcp( socket: &str ) ->

	(TokSplitSink<Framed<TcpStream, MulServTokioCodec<MS>>>, TokSplitStream<Framed<TcpStream, MulServTokioCodec<MS>>>)

{
	// create tcp server
	//
	let socket   = socket.parse::<SocketAddr>().unwrap();
	let listener = TcpListener::bind( &socket ).expect( "bind address" );

	let codec: MulServTokioCodec<MS> = MulServTokioCodec::new();

	let stream   = await!( listener.incoming().take(1).into_future().compat() )
		.expect( "find one stream" ).0
		.expect( "find one stream" );

	codec.framed( stream ).split()
}



pub async fn connect_to_tcp( socket: &str ) -> Addr<MyPeer>
{
	// Connect to tcp server
	//
	let socket = socket.parse::<SocketAddr>().unwrap();
	let stream = await!( TcpStream::connect( &socket ).compat() ).expect( "connect address" );

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
	let peer = Peer::new( addr.clone(), stream_a.compat(), sink_a.sink_compat() ).expect( "spawn peer" );

	mb.start( peer ).expect( "Failed to start mailbox" );

	addr
}



pub async fn connect_return_stream( socket: &str ) ->

	(TokSplitSink<Framed<TcpStream, MulServTokioCodec<MS>>>, TokSplitStream<Framed<TcpStream, MulServTokioCodec<MS>>>)

{
	// Connect to tcp server
	//
	let socket = socket.parse::<SocketAddr>().unwrap();
	let stream = await!( TcpStream::connect( &socket ).compat() ).expect( "connect address" );

	// frame the connection with codec for multiservice
	//
	let codec: MulServTokioCodec<MS> = MulServTokioCodec::new();

	codec.framed( stream ).split()
}




service_map!
(
	namespace:     remote    ;
	peer_type:     MyPeer    ;
	multi_service: MS        ;
	services     : Add, Show ;
);



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
		peer.register_service::<Add >( <Add  as Service<remote::Services>>::sid(), box remote::Services, addr_handler.recipient() );
		peer.register_service::<Show>( <Show as Service<remote::Services>>::sid(), box remote::Services, addr_handler.recipient() );

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


// Test relaying messages
//
#[test]
//
fn relay()
{
	// flexi_logger::Logger::with_str( "remote=trace, thespis_impl=trace, tokio=debug" ).start().unwrap();

	let peera = async
	{
		// get a framed connection
		//
		let (sink_a, stream_a) = await!( listen_tcp( "127.0.0.1:20000" ) );


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
		peer.register_service::<Add >( <Add  as Service<remote::Services>>::sid(), box remote::Services, addr_handler.recipient() );
		peer.register_service::<Show>( <Show as Service<remote::Services>>::sid(), box remote::Services, addr_handler.recipient() );

		mb_peer   .start( peer ).expect( "Failed to start mailbox of Peer" );
	};


	let peerb = async
	{
		let mut peera  = await!( connect_to_tcp( "127.0.0.1:20000" ) );
		let     peera2 = peera.clone();


		// Relay part ---------------------

		let relay = async move
		{
			let (srv_sink, srv_stream) = await!( listen_tcp( "127.0.0.1:30000" ) );


			// Create mailbox for peer
			//
			let mb_peer  : Inbox<MyPeer> = Inbox::new()                  ;
			let peer_addr                = Addr ::new( mb_peer.sender() );

			// create peer with stream/sink + service map
			//
			let mut peer = Peer::new( peer_addr, srv_stream.compat(), srv_sink.sink_compat() ).expect( "spawn peer" );

			peer.register_relayed_service( <Add  as Service<remote::Services>>::sid(), peera2.clone() );
			peer.register_relayed_service( <Show as Service<remote::Services>>::sid(), peera2         );

			await!( mb_peer.start_fut( peer ) );
		};


		let (relay_fut, relay_outcome) = relay.remote_handle();
		rt::spawn( relay_fut ).expect( "failed to spawn server" );

		// --------------------------------------

		let peerc = async
		{
			let mut peerb  = await!( connect_to_tcp( "127.0.0.1:30000" ) );

			// Call the service and receive the response
			//
			let mut add  = remote::Services::recipient::<Add >( peerb.clone() );
			let mut show = remote::Services::recipient::<Show>( peerb.clone() );

			let resp = await!( add.call( Add(5) ) ).expect( "Call failed" );
			assert_eq!( (), resp );

			await!( add.send( Add(5) ) ).expect( "Send failed" );

			let resp = await!( show.call( Show ) ).expect( "Call failed" );
			assert_eq!( 10, resp );

			await!( peerb.send( CloseConnection{ remote: false } ) ).expect( "close connection to peera" );
		};

		// we need to spawn this after peerb, otherwise peerb is not listening yet when we try to connect.
		//
		rt::spawn( peerc ).expect( "Spawn peerc"  );


		// If the peerc closes the connection, close our connection to peera.
		//
		await!( relay_outcome );
		await!( peera.send( CloseConnection{ remote: false } ) ).expect( "close connection to peera" );
	};

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
	fn handle( &mut self, _: Show ) -> Return<u64> { async move
	{
		await!( self.sum.call( Show ) ).expect( "call sum" )

	}.boxed() }
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
		let show = remote::Services::recipient::<Show>( peer_addr.clone() );

		// Create mailbox for our handler
		//
		let addr_handler = Addr::try_from( Parallel{ sum: box show } ).expect( "spawn actor mailbox" );

		// register Sum with peer as handler for Add and Show
		//
		peer.register_service( <Show as Service<parallel::Services>>::sid(), box parallel::Services, addr_handler.recipient() );

		mb_peer   .start( peer ).expect( "Failed to start mailbox of Peer" );
	};


	let peerb = async
	{
		let (sink_b, stream_b) = await!( connect_return_stream( "127.0.0.1:20001" ) );

		// Create mailbox for peer
		//
		let mb_peer      : Inbox<MyPeer> = Inbox::new()                  ;
		let mut peer_addr                = Addr ::new( mb_peer.sender() );

		// create peer with stream/sink
		//
		let mut peer = Peer::new( peer_addr.clone(), stream_b.compat(), sink_b.sink_compat() ).expect( "spawn peer" );

		// Create mailbox for our handler
		//
		let addr_handler = Addr::try_from( Sum(19) ).expect( "spawn actor mailbox" );


		// register Sum with peer as handler for Add and Show
		//
		peer.register_service::<Show>( <Show as Service<remote::Services>>::sid(), box remote::Services, addr_handler.recipient() );

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
