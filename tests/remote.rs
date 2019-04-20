#![ cfg( feature = "remote" ) ]
#![ allow( unused_imports, dead_code ) ]

#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, optin_builtin_traits ) ]

// What is tested:
//
// 1. basic remote functionality
// 2. relays -> TODO
// 3. verify we work in paralell. A calls B, B calls A back for some extra info before answering the first call. -> TODO
// 4. use the same connection for services on both ends -> probably combine this with 3.




use
{
	log          :: { *                                                                         } ,
	thespis      :: { *                                                                         } ,
	thespis_impl :: { single_thread::*, remote::*, service_map, runtime::{ rt, tokio::TokioRT } } ,
	serde        :: { Serialize, Deserialize, de::DeserializeOwned                              } ,
	std          :: { net::SocketAddr                                                           } ,
	bytes        :: { Bytes                                                                     } ,


	futures      ::
	{
		compat :: { Compat01As03, Compat01As03Sink, Stream01CompatExt, Sink01CompatExt, Future01CompatExt } ,
		stream :: { Stream                                                             } ,
	},


	tokio        ::
	{
		await as await01,
		prelude :: { StreamAsyncExt, Stream as TokStream, stream::{ SplitStream as TokSplitStream, SplitSink as TokSplitSink } } ,
 		net     :: { TcpStream, TcpListener                                                                                    } ,
		codec   :: { Decoder, Framed                                                                                           } ,
	},
};


mod common;

use common::import::*;
use common::actors::*;


pub type TheSink = Compat01As03Sink<TokSplitSink<Framed<TcpStream, MulServTokioCodec<MS>>>, MS> ;
pub type MS      = MultiServiceImpl<ServiceID, ConnID, Codecs>                                  ;
pub type MyPeer  = Peer<TheSink, MS>                                                            ;

#[ derive( Serialize, Deserialize, Debug ) ] pub struct ServiceA  { pub msg : String }
#[ derive( Serialize, Deserialize, Debug ) ] pub struct ServiceB  { pub msg : String }


#[ derive( Serialize, Deserialize, Debug ) ]
//
pub struct ResponseA { pub resp: String }


impl Message for ServiceA { type Result = ResponseA; }
impl Message for ServiceB { type Result = ()       ; }

impl Service for ServiceA
{
	type UniqueID = ServiceID;

	fn uid( seed: &[u8] ) -> ServiceID { ServiceID::from_seed( &[ b"ServiceA", seed ].concat() ) }
}

impl Service for ServiceB
{
	type UniqueID = ServiceID;

	fn uid( seed: &[u8] ) -> ServiceID { ServiceID::from_seed( &[ b"ServiceB", seed ].concat() ) }
}


service_map!
(
	namespace:     peer_a   ;
	peer_type:     MyPeer   ;
	multi_service: MS       ;
	send_and_call: ServiceB ;
	call_only:     ServiceA ;
);





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
	let peer = Peer::new( addr.clone(), stream_a.compat(), sink_a.sink_compat() );

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


pub struct HandleA {}

impl Actor for HandleA {}


impl Handler<ServiceA> for HandleA
{
	fn handle( &mut self, _msg: ServiceA ) -> Response<ResponseA> { async move
	{
		ResponseA{ resp: "pong".into() }

	}.boxed() }
}

impl Handler<ServiceB> for HandleA
{
	fn handle( &mut self, _msg: ServiceB ) -> Response<()> { async move
	{
	}.boxed() }
}




impl Service for Add
{
	type UniqueID = ServiceID;

	fn uid( seed: &[u8] ) -> ServiceID { ServiceID::from_seed( &[ b"Add", seed ].concat() ) }
}

impl Service for Show
{
	type UniqueID = ServiceID;

	fn uid( seed: &[u8] ) -> ServiceID { ServiceID::from_seed( &[ b"Show", seed ].concat() ) }
}





service_map!
(
	namespace:     remote   ;
	peer_type:     MyPeer   ;
	multi_service: MS       ;
	send_and_call: Add      ;
	call_only:     Show     ;
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


		// Create mailbox for our actor
		//
		let mb_handler  : Inbox<Sum> = Inbox::new()                     ;
		let addr_handler             = Addr ::new( mb_handler.sender() );

		// Create mailbox for peer
		//
		let mb_peer  : Inbox<MyPeer> = Inbox::new()                  ;
		let peer_addr                = Addr ::new( mb_peer.sender() );

		// create peer with stream/sink
		//
		let mut peer = Peer::new( peer_addr, stream_a.compat(), sink_a.sink_compat() );

		// register Sum with peer as handler for Add and Show
		//
		peer.register_service( Add ::uid( b"remote" ), box remote::Services, addr_handler.recipient::<Add >() );
		peer.register_service( Show::uid( b"remote" ), box remote::Services, addr_handler.recipient::<Show>() );

		let sum = Sum(0);

		mb_peer   .start( peer ).expect( "Failed to start mailbox of Peer" );
		mb_handler.start( sum  ).expect( "Failed to start mailbox for Sum" );
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

		await!( peera.send( CloseConnection ) ).expect( "close connection to peera" );
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


		// Create mailbox for our actor
		//
		let mb_handler  : Inbox<Sum> = Inbox::new()                     ;
		let addr_handler             = Addr ::new( mb_handler.sender() );

		// Create mailbox for peer
		//
		let mb_peer  : Inbox<MyPeer> = Inbox::new()                  ;
		let peer_addr                = Addr ::new( mb_peer.sender() );

		// create peer with stream/sink
		//
		let mut peer = Peer::new( peer_addr, stream_a.compat(), sink_a.sink_compat() );

		// register Sum with peer as handler for Add and Show
		//
		peer.register_service( Add ::uid( b"remote" ), box remote::Services, addr_handler.recipient::<Add >() );
		peer.register_service( Show::uid( b"remote" ), box remote::Services, addr_handler.recipient::<Show>() );

		let sum = Sum(0);

		mb_peer   .start( peer ).expect( "Failed to start mailbox of Peer" );
		mb_handler.start( sum  ).expect( "Failed to start mailbox for Sum" );
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
			let mut peer = Peer::new( peer_addr, srv_stream.compat(), srv_sink.sink_compat() );

			peer.register_relayed_service::<Add >( Add ::uid( b"remote" ), peera2.clone() );
			peer.register_relayed_service::<Show>( Show::uid( b"remote" ), peera2         );

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

			await!( peerb.send( CloseConnection ) ).expect( "close connection to peera" );
		};

		// we need to spawn this after peerb, otherwise peerb is not listening yet when we try to connect.
		//
		rt::spawn( peerc ).expect( "Spawn peerc"  );


		// If the peerc closes the connection, close our connection to peera.
		//
		await!( relay_outcome );
		await!( peera.send( CloseConnection ) ).expect( "close connection to peera" );
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
	fn handle( &mut self, _: Show ) -> Response<u64> { async move
	{
		await!( self.sum.call( Show ) ).expect( "call sum" )

	}.boxed() }
}


service_map!
(
	namespace:     parallel ;
	peer_type:     MyPeer   ;
	multi_service: MS       ;
	send_and_call:          ;
	call_only:     Show     ;
);




// Test correct async behavior. Verify that a peer can continue to
// send/receive while waiting for the response to a call.
//
#[test]
//
fn parallel()
{
	// flexi_logger::Logger::with_str( "remote=trace, thespis_impl=trace, tokio=debug" ).start().unwrap();

	let peera = async
	{
		// get a framed connection
		//
		let (sink_a, stream_a) = await!( listen_tcp( "127.0.0.1:20001" ) );


		// Create mailbox for our actor
		//
		let mb_handler  : Inbox<Parallel> = Inbox::new()                     ;
		let addr_handler                  = Addr ::new( mb_handler.sender() );

		// Create mailbox for peer
		//
		let mb_peer  : Inbox<MyPeer> = Inbox::new()                  ;
		let peer_addr                = Addr ::new( mb_peer.sender() );

		// create peer with stream/sink
		//
		let mut peer = Peer::new( peer_addr.clone(), stream_a.compat(), sink_a.sink_compat() );

		// Create recipients
		//
		let show = remote::Services::recipient::<Show>( peer_addr.clone() );

		// register Sum with peer as handler for Add and Show
		//
		peer.register_service( Show::uid( b"parallel" ), box parallel::Services, addr_handler.recipient::<Show>() );

		let para = Parallel{ sum: box show };

		mb_peer   .start( peer ).expect( "Failed to start mailbox of Peer" );
		mb_handler.start( para ).expect( "Failed to start mailbox for Sum" );
	};


	let peerb = async
	{
		let (sink_b, stream_b) = await!( connect_return_stream( "127.0.0.1:20001" ) );

		// Create mailbox for our actor
		//
		let mb_handler  : Inbox<Sum> = Inbox::new()                     ;
		let addr_handler             = Addr ::new( mb_handler.sender() );

		// Create mailbox for peer
		//
		let mb_peer      : Inbox<MyPeer> = Inbox::new()                  ;
		let mut peer_addr                = Addr ::new( mb_peer.sender() );

		// create peer with stream/sink
		//
		let mut peer = Peer::new( peer_addr.clone(), stream_b.compat(), sink_b.sink_compat() );

		// register Sum with peer as handler for Add and Show
		//
		peer.register_service( Show::uid( b"remote" ), box remote::Services, addr_handler.recipient::<Show>() );

		let sum = Sum(19);

		mb_peer   .start( peer ).expect( "Failed to start mailbox of Peer" );
		mb_handler.start( sum  ).expect( "Failed to start mailbox for Sum" );

		// Create recipients
		//
		let mut show = parallel::Services::recipient::<Show>( peer_addr.clone() );

		let resp = await!( show.call( Show ) ).expect( "Call failed" );
		assert_eq!( 19, resp );

		// dbg!( resp );

		await!( peer_addr.send( CloseConnection ) ).expect( "close connection to peera" );
	};


	rt::spawn( peera  ).expect( "Spawn peera"  );
	rt::spawn( peerb  ).expect( "Spawn peerb"  );

	rt::run();
}
