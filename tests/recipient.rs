#![ feature( optin_builtin_traits ) ]

// TODO:
// - ✔ basic usage (using addr directly uses recipient, so that's already tested)
//     Let's test storing Box<Address<M>> to several actors and send/call on that
//
// - ✔ Receiver
//   - ✔ construct receiver from address and send/call
//   - ✔ cast to Box<Any> and downcast
//
// - ✔ across threads
//
// - ✔ Sink
//   - ✔ send method
//   - ✔ forward stream into it
//   - ✔ on both receiver and addr
//   - use send on &Addr<A> and &Receiver<M>
//
//   when unit tested:
//   - ✔ remove sendr method from recipient?
//   - ✔ clean up all the sync, unpin, 'static we added to make this compile
//   - ✔ some examples with sink?
//
use
{
	std           :: { any::Any, thread                                                       } ,
	futures       :: { channel::oneshot, stream, sink::SinkExt, executor::block_on, StreamExt } ,
	thespis       :: { *                                                                      } ,
	thespis_impl  :: { *                                                                      } ,
	async_executors :: { AsyncStd                                                             } ,
};



#[ derive( Actor ) ] struct MyActor{ count: u8 }
#[ derive( Actor ) ] struct Other  { count: u8 }

struct Count;


impl Message for Count
{
	type Return = u8;
}


impl Handler< Count > for MyActor
{
	fn handle( &mut self, _msg: Count ) -> Return<u8> { Box::pin( async move
	{
		self.count += 1;
		self.count

	})}
}


impl Handler< Count > for Other
{
	fn handle( &mut self, _msg: Count ) -> Return<u8> { Box::pin( async move
	{
		self.count += 1;
		self.count

	})}
}



// Verify we can box up recipients to different actors in one vector and use them.
//
#[ test ]
//
fn store_recipients()
{
	let program = async move
	{
		let a = MyActor { count: 0 };
		let b = Other   { count: 0 };

		let exec = AsyncStd{};

		let addr  = Addr::builder().start( a, &exec ).expect( "Failed to create address" );
		let addro = Addr::builder().start( b, &exec ).expect( "Failed to create address" );


		let mut recs: Vec<Box< dyn Address<Count, Error=ThesErr> >> = vec![ Box::new( addr ), Box::new( addro ) ];

		recs[ 0 ].send( Count ).await.expect( "Send failed" );
		recs[ 1 ].send( Count ).await.expect( "Send failed" );
		recs[ 0 ].send( Count ).await.expect( "Send failed" );

		assert_eq!( 3,  recs[ 0 ].call( Count ).await.expect( "Call failed" ) );
		assert_eq!( 2,  recs[ 1 ].call( Count ).await.expect( "Call failed" ) );
	};

	block_on( program );
}



// Use the Receiver struct (basic usage)
//
#[ test ]
//
fn receiver_basic_use()
{
	let program = async move
	{
		let a = MyActor { count: 0 };
		let b = Other   { count: 0 };

		let exec = AsyncStd{};

		let addr  = Addr::builder().start( a, &exec ).expect( "Failed to create address" );
		let addro = Addr::builder().start( b, &exec ).expect( "Failed to create address" );


		let mut recs: Vec< Receiver<Count> > = vec![ Receiver::new( Box::new( addr ) ), Receiver::new( Box::new( addro ) ) ];

		recs[ 0 ].send( Count ).await.expect( "Send failed" );
		recs[ 1 ].send( Count ).await.expect( "Send failed" );
		recs[ 0 ].send( Count ).await.expect( "Send failed" );

		assert_eq!( 3,  recs[ 0 ].call( Count ).await.expect( "Call failed" ) );
		assert_eq!( 2,  recs[ 1 ].call( Count ).await.expect( "Call failed" ) );
	};

	block_on( program );
}



// Verify we can box Receiver as Box<Any> and downcast it.
//
#[ test ]
//
fn receiver_box_any()
{
	let program = async move
	{
		let a = MyActor { count: 0 };
		let b = Other   { count: 0 };

		let exec = AsyncStd{};

		let addr  = Addr::builder().start( a, &exec ).expect( "Failed to create address" );
		let addro = Addr::builder().start( b, &exec ).expect( "Failed to create address" );


		let recs: Vec< Box<dyn Any> > = vec!
		[
			Box::new( Receiver::new( Box::new( addr  ) ) ),
			Box::new( Receiver::new( Box::new( addro ) ) ),
		];


		let mut reca = recs[ 0 ].downcast_ref::<Receiver<Count>>().expect( "downcast" ).clone();
		let mut recb = recs[ 1 ].downcast_ref::<Receiver<Count>>().expect( "downcast" ).clone();

		reca.send( Count ).await.expect( "Send failed" );
		recb.send( Count ).await.expect( "Send failed" );
		reca.send( Count ).await.expect( "Send failed" );

		assert_eq!( 3,  reca.call( Count ).await.expect( "Call failed" ) );
		assert_eq!( 2,  recb.call( Count ).await.expect( "Call failed" ) );
	};

	block_on( program );
}



// Use Recipient across threads
//
#[ test ]
//
fn multi_thread()
{
	let program = async move
	{
		let a = MyActor { count: 0 };
		let b = Other   { count: 0 };

		let exec = AsyncStd{};

		// This will spawn the task for the mailbox on the current thread
		//
		let addr  = Addr::builder().start( a, &exec ).expect( "Failed to create address" );
		let addro = Addr::builder().start( b, &exec ).expect( "Failed to create address" );

		let mut reca: Vec<Box< dyn Address<Count, Error=ThesErr> >> = vec![ Box::new( addr.clone() ), Box::new( addro.clone() ) ];
		let mut recb: Vec<Box< dyn Address<Count, Error=ThesErr> >> = vec![ Box::new( addr )        , Box::new( addro )         ];

		let (tx, rx) = oneshot::channel::<()>();


		thread::spawn( move ||
		{
			let thread_program = async move
			{
				reca[ 0 ].send( Count ).await.expect( "Send failed" );
				reca[ 1 ].send( Count ).await.expect( "Send failed" );
				reca[ 0 ].send( Count ).await.expect( "Send failed" );
			};

			block_on( thread_program );

			tx.send(()).expect( "Signal end of thread" );

		});

		// TODO: create a way to join threads asynchronously...
		//
		rx.await.expect( "receive Signal end of thread" );


		assert_eq!( 3,  recb[0].call( Count ).await.expect( "Call failed" ) );
		assert_eq!( 2,  recb[1].call( Count ).await.expect( "Call failed" ) );
	};

	block_on( program );
}




// Use send_all on a Address to forward all messages from a stream to Addr<A>
//
#[ test ]
//
fn stream_to_sink_addr()
{
	let program = async move
	{
		let a = MyActor { count: 0 };
		let exec = AsyncStd{};

		let mut addr    = Addr::builder().start( a, &exec ).expect( "Failed to create address" );
		let mut stream  = stream::iter( vec![ Count, Count, Count ].into_iter() ).map( |i| Ok(i) );

		addr.send_all( &mut stream ).await.expect( "drain stream" );

		// This doesn't really work:
		// - stream needs to be a TryStream
		// - the future will only complete when the sink is closed, but our addresses can only
		//   close when they are dropped.
		//
		// let mut stream2 = stream::iter( vec![ Count, Count, Count ].into_iter() );
		//  stream.forward( &mut addr ).await.expect( "forward to sink" );

		assert_eq!( 4,  addr.call( Count ).await.expect( "Call failed" ) );
	};

	block_on( program );
}



// Use send_all on a Recipient to forward all messages from a stream to Receiver<M>
//
#[ test ]
//
fn stream_to_sink_receiver()
{
	let program = async move
	{
		let a = MyActor { count: 0 };
		let exec = AsyncStd{};

		let addr        = Addr::builder().start( a, &exec ).expect( "Failed to create address" );
		let mut clone   = Receiver::new( Address::clone_box(&addr) );
		let mut stream  = stream::iter( vec![ Count, Count, Count ].into_iter() ).map( |i| Ok(i) );

		clone.send_all( &mut stream ).await.expect( "drain stream" );

		// This doesn't really work:
		// - stream needs to be a TryStream
		// - the future will only complete when the sink is closed, but our addresses can only
		//   close when they are dropped.
		//
		// let mut stream2 = stream::iter( vec![ Count, Count, Count ].into_iter() );
		//  stream.forward( &mut addr ).await.expect( "forward to sink" );

		assert_eq!( 4,  clone.call( Count ).await.expect( "Call failed" ) );
	};

	block_on( program );
}



// Verify we can box up recipients to different actors in one vector and use them.
//
#[ test ]
//
fn actor_id()
{
	let program = async move
	{
		let a = MyActor { count: 0 };
		let b = MyActor { count: 0 };

		let addr  = Addr::builder().start( a, &AsyncStd ).expect( "Failed to create address" );
		let addrb = Addr::builder().start( b, &AsyncStd ).expect( "Failed to create address" );
		let rec   = Address::clone_box( &addr );

		// return same value on subsequent calls
		//
		assert_eq!( addr.id(), addr.id() );

		// return same value on clone
		//
		assert_eq!( addr.id(), addr.clone().id() );

		// return same value on Box<Address<_>>
		//
		assert_eq!( addr.id(), rec.id() );

		// return different value for different actor
		//
		assert_ne!( addr.id(), addrb.id() );
	};

	block_on( program );
}
