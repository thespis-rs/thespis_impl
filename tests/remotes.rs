#![ cfg( feature = "remote" ) ]

#![ feature( await_macro, async_await, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, optin_builtin_traits ) ]

// What is tested:
//
// - ✔ basic remote functionality
// - ✔ relays
// - ✔ verify we work in paralell. A calls B, B calls A back for some extra info before answering the first call.
// - ✔ use the same connection for services on both ends -> probably combine this with 3.
//
// TODO:
// - call remote service after connection closed. Verify error returned
//
// - decide on relaying in runtime, without having the relayed types at compilation
// - stop relaying if a relay goes down, afterwards bring it back up and relay again?
//
// - verify all events on the events stream are outputted correctly
// - verify eventstream end as detection of connection close
//
// - return errors to remote.(still need to implement receiving end for error)
//
// - ✔ test service map: 2 different namespaces should give different sid, ...




pub use
{
	thespis      :: { *                                          } ,
	thespis_impl :: { *, remote::*, service_map, runtime::{ rt } } ,
	std          :: { net::SocketAddr                            } ,
	pharos       :: { Observable                                 } ,


	futures      ::
	{
		channel :: { mpsc                                                                    } ,
		compat  :: { Compat01As03Sink, Stream01CompatExt, Sink01CompatExt, Future01CompatExt } ,
		prelude :: { StreamExt                                                               } ,
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


mod remote;



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




pub async fn connect_to_tcp( socket: &str ) -> (Addr<MyPeer>, mpsc::Receiver<PeerEvent>)
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
	let mut peer = Peer::new( addr.clone(), stream_a.compat(), sink_a.sink_compat() ).expect( "spawn peer" );

	let evts = peer.observe( 10 );

	mb.start( peer ).expect( "Failed to start mailbox" );

	(addr, evts)
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
	namespace:     remotes   ;
	peer_type:     MyPeer    ;
	multi_service: MS        ;
	services     : Add, Show ;
);






