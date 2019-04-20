pub use
{
	log          :: { *                                                                         } ,
	thespis      :: { *                                                                         } ,
	thespis_impl :: { single_thread::*, remote::*, service_map, runtime::{ rt, tokio::TokioRT } } ,
	serde        :: { Serialize, Deserialize, de::DeserializeOwned                              } ,
	std          :: { net::SocketAddr                                                           } ,
	bytes        :: { Bytes                                                                     } ,


	futures      ::
	{
		future :: { FutureExt                                                          } ,
		compat :: { Future01CompatExt, Compat01As03, Compat01As03Sink, Stream01CompatExt, Sink01CompatExt } ,
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



// compiler bug
//
#[ allow( dead_code ) ]
//
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



// compiler bug
//
#[ allow( dead_code ) ]
//
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

