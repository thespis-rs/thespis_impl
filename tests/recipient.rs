// TODO:
// - ✔ basic usage (using addr directly uses recipient, so that's already tested)
//     Let's test storing Box<Recipient<M>> to several actors and send/call on that
//
// - ✔ Receiver
//   - ✔ construct receiver from address and send/call
//   - ✔ cast to Box<Any> and downcast
//
// - ✔ across threads
// - ✔ Sink
//   - ✔ send method
//   - ✔ forward stream into it
//   - ✔ on both receiver and addr
//
//   when unit tested:
//   - ✔ remove sendr method from recipient?
//   - clean up all the sync, unpin, 'static we added to make this compile
//   - some examples with sink?
//   - error handling on peer with the new send interface
//     - unit test peer error handling
//

#![ feature( await_macro, async_await, arbitrary_self_types, box_syntax, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

use
{
	std           :: { any::Any, thread                                           } ,
	futures       :: { future::FutureExt, channel::oneshot, stream, sink::SinkExt } ,
	thespis       :: { *                                                          } ,
	thespis_impl  :: { *, runtime::rt                                             } ,
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
	fn handle( &mut self, _msg: Count ) -> Return<u8> { async move
	{
		self.count += 1;
		self.count

	}.boxed() }
}


impl Handler< Count > for Other
{
	fn handle( &mut self, _msg: Count ) -> Return<u8> { async move
	{
		self.count += 1;
		self.count

	}.boxed() }
}



#[ test ]
//
fn store_recipients()
{
	let program = async move
	{
		let a = MyActor { count: 0 };
		let b = Other   { count: 0 };

		let addr  = Addr::try_from( a ).expect( "Failed to create address" );
		let addro = Addr::try_from( b ).expect( "Failed to create address" );


		let mut recs: Vec<Box< Recipient<Count> >> = vec![ box addr, box addro ];

		await!( recs[ 0 ].send( Count ) ).expect( "Send failed" );
		await!( recs[ 1 ].send( Count ) ).expect( "Send failed" );
		await!( recs[ 0 ].send( Count ) ).expect( "Send failed" );

		assert_eq!( 3, await!( recs[ 0 ].call( Count ) ).expect( "Call failed" ) );
		assert_eq!( 2, await!( recs[ 1 ].call( Count ) ).expect( "Call failed" ) );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}



#[ test ]
//
fn receiver_basic_use()
{
	let program = async move
	{
		let a = MyActor { count: 0 };
		let b = Other   { count: 0 };

		let addr  = Addr::try_from( a ).expect( "Failed to create address" );
		let addro = Addr::try_from( b ).expect( "Failed to create address" );


		let mut recs: Vec< Receiver<Count> > = vec![ Receiver::new( box addr ), Receiver::new( box addro ) ];

		await!( recs[ 0 ].send( Count ) ).expect( "Send failed" );
		await!( recs[ 1 ].send( Count ) ).expect( "Send failed" );
		await!( recs[ 0 ].send( Count ) ).expect( "Send failed" );

		assert_eq!( 3, await!( recs[ 0 ].call( Count ) ).expect( "Call failed" ) );
		assert_eq!( 2, await!( recs[ 1 ].call( Count ) ).expect( "Call failed" ) );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}



#[ test ]
//
fn receiver_box_any()
{
	let program = async move
	{
		let a = MyActor { count: 0 };
		let b = Other   { count: 0 };

		let addr  = Addr::try_from( a ).expect( "Failed to create address" );
		let addro = Addr::try_from( b ).expect( "Failed to create address" );


		let recs: Vec< Box<Any> > = vec![ Box::new( Receiver::new( box addr ) ), Box::new( Receiver::new( box addro ) ) ];


		let mut reca = recs[ 0 ].downcast_ref::<Receiver<Count>>().expect( "downcast" ).clone();
		let mut recb = recs[ 1 ].downcast_ref::<Receiver<Count>>().expect( "downcast" ).clone();

		await!( reca.send( Count ) ).expect( "Send failed" );
		await!( recb.send( Count ) ).expect( "Send failed" );
		await!( reca.send( Count ) ).expect( "Send failed" );

		assert_eq!( 3, await!( reca.call( Count ) ).expect( "Call failed" ) );
		assert_eq!( 2, await!( recb.call( Count ) ).expect( "Call failed" ) );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}



#[ test ]
//
fn multi_thread()
{
	let program = async move
	{
		let a = MyActor { count: 0 };
		let b = Other   { count: 0 };

		// This will spawn the task for the mailbox on the current thread
		//
		let addr  = Addr::try_from( a ).expect( "Failed to create address" );
		let addro = Addr::try_from( b ).expect( "Failed to create address" );

		let mut reca: Vec<Box< Recipient<Count> >> = vec![ box addr.clone(), box addro.clone() ];
		let mut recb: Vec<Box< Recipient<Count> >> = vec![ box addr        , box addro         ];

		let (tx, rx) = oneshot::channel::<()>();


		thread::spawn( move ||
		{
			let thread_program = async move
			{
				await!( reca[ 0 ].send( Count ) ).expect( "Send failed" );
				await!( reca[ 1 ].send( Count ) ).expect( "Send failed" );
				await!( reca[ 0 ].send( Count ) ).expect( "Send failed" );
			};

			rt::spawn( thread_program ).expect( "Spawn thread program" );
			rt::run();

			tx.send(()).expect( "Signal end of thread" );

		});

		// TODO: create a way to join threads asynchronously...
		//
		await!( rx ).expect( "receive Signal end of thread" );


		assert_eq!( 3, await!( recb[0].call( Count ) ).expect( "Call failed" ) );
		assert_eq!( 2, await!( recb[1].call( Count ) ).expect( "Call failed" ) );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}




#[ test ]
//
fn stream_to_sink_addr()
{
	let program = async move
	{
		let a = MyActor { count: 0 };

		let mut addr    = Addr::try_from( a ).expect( "Failed to create address" );
		let mut stream  = stream::iter( vec![ Count, Count, Count ].into_iter() );

		await!( addr.send_all( &mut stream ) ).expect( "drain stream" );

		// This doesn't really work:
		// - stream needs to be a TryStream
		// - the future will only complete when the sink is closed, but our addresses can only
		//   close when they are dropped.
		//
		// let mut stream2 = stream::iter( vec![ Count, Count, Count ].into_iter() );
		// await!( stream.forward( &mut addr ) ).expect( "forward to sink" );

		assert_eq!( 4, await!( addr.call( Count ) ).expect( "Call failed" ) );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}




#[ test ]
//
fn stream_to_sink_receiver()
{
	let program = async move
	{
		let a = MyActor { count: 0 };

		let addr        = Addr::try_from( a ).expect( "Failed to create address" );
		let mut rec     = Receiver::new( addr.clone_box() );
		let mut stream  = stream::iter( vec![ Count, Count, Count ].into_iter() );

		await!( rec.send_all( &mut stream ) ).expect( "drain stream" );

		// This doesn't really work:
		// - stream needs to be a TryStream
		// - the future will only complete when the sink is closed, but our addresses can only
		//   close when they are dropped.
		//
		// let mut stream2 = stream::iter( vec![ Count, Count, Count ].into_iter() );
		// await!( stream.forward( &mut addr ) ).expect( "forward to sink" );

		assert_eq!( 4, await!( rec.call( Count ) ).expect( "Call failed" ) );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}
