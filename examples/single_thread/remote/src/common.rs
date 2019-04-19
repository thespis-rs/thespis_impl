pub use
{
	log          :: { *                                                                                } ,
	thespis      :: { *                                                                                } ,
	thespis_impl :: { single_thread::*, remote::*, runtime::{ rt, tokio::TokioRT }                               } ,
	tokio        :: { await as await01, prelude::{ StreamAsyncExt, Stream as TokStream, stream::SplitStream as TokSplitStream, stream::SplitSink as TokSplitSink }, net::{ TcpStream, TcpListener }, codec::{ Decoder, Framed } } ,
	serde        :: { Serialize, Deserialize, de::DeserializeOwned                                     } ,
	serde_cbor   :: { from_slice as des                                                                } ,
	std          :: { net::SocketAddr, any::Any                                                        } ,
	futures      :: { future::{ FutureExt }, compat::{ Compat01As03, Compat01As03Sink, Stream01CompatExt, Sink01CompatExt }, stream::{ Stream } } ,
	bytes        :: { Bytes },
};



pub type TheSink = Compat01As03Sink<TokSplitSink<Framed<TcpStream, MulServTokioCodec<MS>>>, MS>;
pub type MS      = MultiServiceImpl<ServiceID, ConnID, Codecs>;
pub type MyPeer  = Peer<TheSink, MS>;

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
	fn sid()              -> ServiceID { ServiceID::from_seed(    b"ServiceA"                  ) }
}

impl Service for ServiceB
{
	type UniqueID = ServiceID;

	fn uid( seed: &[u8] ) -> ServiceID { ServiceID::from_seed( &[ b"ServiceB", seed ].concat() ) }
	fn sid()              -> ServiceID { ServiceID::from_seed(    b"ServiceB"                  ) }
}






pub async fn listen_tcp( socket: &str ) -> (TokSplitSink<Framed<TcpStream, MulServTokioCodec<MS>>>, TokSplitStream<Framed<TcpStream, MulServTokioCodec<MS>>>)
{
	// create tcp server
	//
	let socket   = socket.parse::<SocketAddr>().unwrap();
	let listener = TcpListener::bind( &socket ).expect( "bind address" );

	let codec: MulServTokioCodec<MS> = MulServTokioCodec::new();

	let stream   = await01!( listener.incoming().take(1).into_future() )
		.expect( "find one stream" ).0
		.expect( "find one stream" );

	codec.framed( stream ).split()
}




pub async fn connect_to_tcp( socket: &str ) -> Addr<MyPeer>
{
	// Connect to tcp server
	//
	let socket = socket.parse::<SocketAddr>().unwrap();
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

	addr
}












pub mod peer_a
{

use super::*;

// Mark the services part of this particular service map, so we can write generic impls only for them.
//
pub trait MarkServices {}

impl MarkServices for ServiceA {}
impl MarkServices for ServiceB {}


#[ derive( Clone ) ]
//
pub struct Services;

impl Services
{
	pub fn recipient<S>( peer: Addr<MyPeer> ) -> impl Recipient<S>

	where  S                    : MarkServices + Service<UniqueID=ServiceID> + Serialize + DeserializeOwned ,
	      <S as Message>::Result: Serialize + DeserializeOwned                                              ,
	{
		ServicesRecipient::new( peer )
	}


	fn send_service_gen<S>( msg: MS, receiver: &Box< dyn Any > )

		where S                     : Service + Message<Result=()>,
		      <S as Message>::Result: Serialize + DeserializeOwned,

	{
		let     backup: &Rcpnt<S> = receiver.downcast_ref().expect( "downcast_ref failed" );
		let mut rec               = backup.clone_box();

		let message = serde_cbor::from_slice( &msg.mesg() ).expect( "deserialize serviceA" );

		rt::spawn( async move
		{
			await!( rec.send( message ) ).expect( "call actor" );

		}).expect( "spawn call for service" );
	}



	fn call_service_gen<S>
	(
		     msg        :  MS                        ,
		     receiver   : &Box< dyn Any >            ,
		 mut return_addr:  Box< dyn Recipient< MS >> ,

	)

	where S                     : Service                     ,
	      <S as Message>::Result: Serialize + DeserializeOwned,
	{
		let     backup: &Rcpnt<S> = receiver.downcast_ref().expect( "downcast_ref failed" );
		let mut rec               = backup.clone_box();

		let message = serde_cbor::from_slice( &msg.mesg() ).expect( "deserialize serviceA" );

		rt::spawn( async move
		{
			let resp       = await!( rec.call( message ) ).expect( "call actor" );

			let sid        = ServiceID::null();
			let cid        = msg.conn_id().expect( "get conn_id" );
			let serialized = serde_cbor::to_vec( &resp ).expect( "serialize response" );

			let mul        = MultiServiceImpl::new( sid, cid, Codecs::CBOR, serialized.into() );

			await!( return_addr.send( mul ) ).expect( "send response back to peer" );

		}).expect( "spawn call for service" );
	}
}


impl ServiceMap<MS> for Services
{
	fn send_service( &self, msg: MS, receiver: &Box< dyn Any > )
	{
		let x = msg.service().expect( "get service" );

		match x
		{
			_ if x == ServiceB::sid() => Self::send_service_gen::<ServiceB>( msg, receiver ),
			_                         => panic!( "got wrong service: {:?}", x ),
		}
	}


	fn call_service
	(
		&self                                    ,
		 msg        :  MS                        ,
		 receiver   : &Box< dyn Any >            ,
		 return_addr:  Box< dyn Recipient< MS >> ,

	)
	{
		let x = msg.service().expect( "get service" );

		match x
		{
			_ if x == ServiceA::sid() => Self::call_service_gen::<ServiceA>( msg, receiver, return_addr ) ,
			_ if x == ServiceB::sid() => Self::call_service_gen::<ServiceB>( msg, receiver, return_addr ) ,
			_                         => panic!( "got wrong service: {:?}", x )                           ,
		}
	}
}



#[ derive( Clone ) ]
//
pub struct ServicesRecipient
{
	peer: Addr<MyPeer>
}


impl ServicesRecipient
{
	pub fn new( peer: Addr<MyPeer> ) -> Self
	{
		Self { peer }
	}


	async fn send_gen<S>( &mut self, msg: S ) -> ThesRes<()>

		where  S                    : Service<UniqueID=ServiceID> ,
				<S as Message>::Result: Serialize + DeserializeOwned,

	{
		let sid        = S::sid();
		let cid        = ConnID::null();
		let serialized = serde_cbor::to_vec( &msg )?;

		let mul        = MultiServiceImpl::new( sid, cid, Codecs::CBOR, serialized.into() );

		await!( self.peer.send( mul ) )
	}


	async fn call_gen<S>( &mut self, msg: S ) -> ThesRes<<S as Message>::Result>

		where  S                    : Service<UniqueID=ServiceID> ,
				<S as Message>::Result: Serialize + DeserializeOwned,

	{
		let sid        = S::sid();
		let cid        = ConnID::default();
		let serialized = serde_cbor::to_vec( &msg )?.into();

		let mul        = MultiServiceImpl::new( sid, cid, Codecs::CBOR, serialized );

		let call = Call::new( mul );
		let re   = await!( await!( self.peer.call( call ) )? )?;

		Ok( des( &re.mesg() )? )
	}
}




impl<S> Recipient<S> for ServicesRecipient

	where  S                    : MarkServices + Service<UniqueID=ServiceID> + Serialize + DeserializeOwned ,
	      <S as Message>::Result: Serialize + DeserializeOwned                                              ,

{
	fn send( &mut self, msg: S ) -> Response< ThesRes<()> >
	{
		self.send_gen( msg ).boxed()
	}


	fn call( &mut self, msg: S ) -> Response< ThesRes<<S as Message>::Result> >
	{
		self.call_gen( msg ).boxed()
	}


	fn clone_box( &self ) -> Box< dyn Recipient<S> >
	{
		box Self { peer: self.peer.clone() }
	}
}

}
