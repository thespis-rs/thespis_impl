#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax ) ]


use
{
	std          :: { net::SocketAddr                                                                     } ,
	futures      :: { future::{ FutureExt }                                                               } ,
	thespis      :: { *                                                                                   } ,
	thespis_impl :: { single_thread::*,  runtime::{ rt, tokio::TokioRT }                                  } ,
	tokio        :: { codec::{ LinesCodec, Decoder, Framed }                                              } ,
	tokio        :: { prelude::{ Stream as Stream01, Sink as Sink01, stream::SplitStream }                } ,
	tokio        :: { await as await01, prelude::{ StreamAsyncExt }, net::{ TcpStream, TcpListener },     } ,
};


#[ derive( Actor ) ]
//
struct Listener
{
	stream: SplitStream<Framed<TcpStream, LinesCodec>>
}


struct Next;

impl Message for Next
{
	type Result = Option<String>;
}


impl Listener
{
	fn new( stream: SplitStream<Framed<TcpStream, LinesCodec>> ) -> Self
	{
		Self { stream }
	}
}


impl Handler< Next > for Listener
{
	fn handle( &mut self, _msg: Next ) -> Response<Option< String >>
	{
		async move
		{
			match await01!( self.stream.next() )
			{
				Some( res ) =>
				{
					match res
					{
						Ok ( s ) => Some( s ),
						Err( e ) => Err ( e ).expect( "failed to read from stream" )
					}
				},

				None => None
			}

		}.boxed()
	}
}



fn main()
{
	rt::init( box TokioRT::default() ).expect( "We only set the executor once" );

	// Create mailbox
	//
	let     mb  : Inbox<Listener> = Inbox::new();
	let mut addr                  = Addr::new( mb.sender() );


	let listen = async move
	{
		let addr     = "127.0.0.1:8998".parse::<SocketAddr>().unwrap();
		let listener = TcpListener::bind( &addr ).expect( "bind address" );
		let codec    = LinesCodec::new();
		let stream   = await01!( listener.incoming().take(1).into_future() ).expect( "find one stream" ).0.expect( "find one stream" );

		let (_     , stream_b) = codec.framed( stream ).split();
		let incoming = Listener::new( stream_b );

		mb.start( incoming ).expect( "Failed to start mailbox" );
	};

	let connect = async move
	{
		let socket = "127.0.0.1:8998".parse::<SocketAddr>().unwrap();
		let stream = await01!( TcpStream::connect( &socket ) ).expect( "connect address" );
		let codec  = LinesCodec::new();

		// Get read and write parts of our streams
		//
		let (sink_a, _       ) = codec.framed( stream ).split();


		let test = "HAHAHA".to_string();

		await01!( sink_a.send( test.clone() ) ).expect( "sending failed" );

		if let Some(s) = await!( addr.call( Next ) )
		{
			// You should see the debug output to prove that this runs and it's working.
			//
			assert_eq!( dbg!( s ), test );
		}
	};

	rt::spawn( listen  ).expect( "Spawn listen"  );
	rt::spawn( connect ).expect( "Spawn connect" );
	rt::run();
}
