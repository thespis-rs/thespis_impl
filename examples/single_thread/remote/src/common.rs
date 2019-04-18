pub use
{
	log          :: { *                                                                                } ,
	thespis      :: { *                                                                                } ,
	thespis_impl :: { single_thread::*, remote::*, runtime::{ rt, tokio::TokioRT }                               } ,
	tokio        :: { await as await01, prelude::{ StreamAsyncExt, Stream as TokStream, stream::SplitSink as TokSplitSink }, net::{ TcpStream, TcpListener }, codec::{ Decoder, Framed } } ,
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

	#[ inline ]
	//
	fn uid( seed: &[u8] ) -> ServiceID
	{
		ServiceID::from_seed( &[ b"ServiceA", seed ].concat() )
	}

	#[ inline ]
	//
	fn sid() -> ServiceID
	{
		ServiceID::from_seed( b"ServiceA" )
	}
}

impl Service for ServiceB
{
	type UniqueID = ServiceID;

	#[ inline ]
	//
	fn uid( seed: &[u8] ) -> ServiceID
	{
		ServiceID::from_seed( &[ b"ServiceB", seed ].concat() )
	}

	#[ inline ]
	//
	fn sid() -> ServiceID
	{
		ServiceID::from_seed( b"ServiceB" )
	}
}



#[ derive( Clone ) ]
//
pub struct PeerAServices;

impl PeerAServices
{
	pub fn recip_service_a( peer: Addr<MyPeer> ) -> impl RemoteRecipient<ServiceA>
	{
		PeerAServicesRecipient::new( peer )
	}

	pub fn recip_service_b( peer: Addr<MyPeer> ) -> impl RemoteRecipient<ServiceB>
	{
		PeerAServicesRecipient::new( peer )
	}
}


impl ServiceMap<MS> for PeerAServices
{
	fn deserialize( &self )
	{

	}


	fn send_service
	(
		&self                     ,
		 msg     : MS             ,
		 receiver: &Box< dyn Any >,

	)
	{
		let service_a = ServiceID::from_seed( b"ServiceB" );

		if msg.service().expect( "get service" ) == service_a
		{
			let     backup: &Rcpnt<ServiceB> = receiver.downcast_ref().expect( "downcast_ref failed" );
			let mut rec                      = backup.clone_box();

			let message = serde_cbor::from_slice( &msg.mesg() ).expect( "deserialize serviceA" );

			rt::spawn( async move
			{
				await!( rec.send( message ) ).expect( "call actor" );

			}).expect( "spawn call for servicea" );
		}

		else
		{
			panic!( "got wrong service" );
		}
	}

	fn call_service
	(
		&self                                   ,
		 msg        : MS                        ,
		 receiver   : &Box< dyn Any > ,
		 mut return_addr: Box< dyn Recipient< MS >> ,

	)
	{
		let service_a = ServiceID::from_seed( b"ServiceA" );

		if msg.service().expect( "get service" ) == service_a
		{
			let     backup: &Rcpnt<ServiceA> = receiver.downcast_ref().expect( "downcast_ref failed" );
			let mut rec                      = backup.clone_box();

			let message = serde_cbor::from_slice( &msg.mesg() ).expect( "deserialize serviceA" );

			rt::spawn( async move
			{
				let resp = await!( rec.call( message ) ).expect( "call actor" );

				let sid = ServiceID::null();
				let cid = msg.conn_id().expect( "get conn_id" );

				let serialized = serde_cbor::to_vec( &resp ).expect( "serialize response" );

				let mul = MultiServiceImpl::new( sid, cid, Codecs::CBOR, Bytes::from( serialized ) );

				await!( return_addr.send( mul ) ).expect( "send response back to peer" );

			}).expect( "spawn call for servicea" );
		}

		else
		{
			panic!( "got wrong service" );
		}
	}
}



#[ derive( Clone ) ]
//
pub struct PeerAServicesRecipient
{
	peer: Addr<MyPeer>
}


impl PeerAServicesRecipient
{
	pub fn new( peer: Addr<MyPeer> ) -> Self
	{
		Self { peer }
	}
}


impl RemoteRecipient<ServiceA> for PeerAServicesRecipient
{
	fn send( &mut self, msg: ServiceA ) -> Response< ThesRes<()> >
	{
		async move
		{
			let sid = ServiceA::sid();
			let cid = ConnID::null();

			let serialized = serde_cbor::to_vec( &msg )?;

			let mul = MultiServiceImpl::new( sid, cid, Codecs::CBOR, Bytes::from( serialized ) );

			await!( self.peer.send( mul ) )

		}.boxed()
	}


	fn call( &mut self, msg: ServiceA ) -> Response< ThesRes<<ServiceA as Message>::Result> >
	{
		async move
		{
			let sid = ServiceA::sid();
			let cid = ConnID::default();

			dbg!( &sid );

			let serialized = serde_cbor::to_vec( &msg )?.into();

			let mul  = MultiServiceImpl::new( sid, cid, Codecs::CBOR, serialized );

			let call = Call::new( mul );
			dbg!( "RemoteRecipient: Sending message to peer" );

			let re   = await!( await!( self.peer.call( call ) )? )?;

			let resp: <ServiceA as Message>::Result = des( &re.mesg() )?;

			Ok( resp )

		}.boxed()
	}


	fn clone_box( &self ) -> Box< dyn RemoteRecipient<ServiceA> >
	{
		box Self { peer: self.peer.clone() }
	}
}


impl RemoteRecipient<ServiceB> for PeerAServicesRecipient
{
	fn send( &mut self, msg: ServiceB ) -> Response< ThesRes<()> >
	{
		async move
		{
			let sid = ServiceB::sid();
			let cid = ConnID::null();

			let serialized = serde_cbor::to_vec( &msg )?;

			let mul = MultiServiceImpl::new( sid, cid, Codecs::CBOR, Bytes::from( serialized ) );

			await!( self.peer.send( mul ) )

		}.boxed()
	}


	fn call( &mut self, msg: ServiceB ) -> Response< ThesRes<<ServiceB as Message>::Result> >
	{
		async move
		{
			let sid = ServiceB::sid();
			let cid = ConnID::default();

			dbg!( &sid );

			let serialized = serde_cbor::to_vec( &msg )?.into();

			let mul  = MultiServiceImpl::new( sid, cid, Codecs::CBOR, serialized );

			let call = Call::new( mul );
			dbg!( "RemoteRecipient: Sending message to peer" );

			let re   = await!( await!( self.peer.call( call ) )? )?;

			let resp: <ServiceB as Message>::Result = des( &re.mesg() )?;

			Ok( resp )

		}.boxed()
	}


	fn clone_box( &self ) -> Box< dyn RemoteRecipient<ServiceB> >
	{
		box Self { peer: self.peer.clone() }
	}
}
