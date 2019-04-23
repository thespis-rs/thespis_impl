#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

#![ allow( dead_code, unused_imports )]


use
{
	futures       :: { future::{ Future, FutureExt }, task::{ LocalSpawn, SpawnExt, LocalSpawnExt }, executor::LocalPool  } ,
	std           :: { pin::Pin                                                                            } ,
	log           :: { *                                                                                   } ,
	thespis       :: { *                                                                                   } ,
	thespis_impl  :: { single_thread::*, runtime::rt                                                       } ,
};


#[ derive( Actor ) ]
//
struct MyActor { seed: String }

impl MyActor
{
	async fn bla( x: &mut String ) -> String
	{
		x.extend( "bla".chars() );
		x.clone()
	}
}


struct Ping( String );


impl Message for Ping
{
	type Result = String;
}


impl Handler< Ping > for MyActor
{
	// If you forget the move on the end, it won't compile and error messages will be shit!!!
	//
	fn handle( &mut self, msg: Ping ) -> Return<String> { async move
	{
		trace!( "Ping handler called" );

		self.seed.extend( msg.0.chars() );

		self.seed = await!( Self::bla( &mut self.seed ) );

		self.seed += " - after yield";

		self.seed.clone()

	}.boxed() }
}



fn main()
{
	simple_logger::init().unwrap();

	let program = async move
	{
		let a = MyActor{ seed: "seed".into() };

		// Create mailbox
		//
		let mb  : Inbox<MyActor> = Inbox::new();
		let send                 = mb.sender ();

		mb.start( a ).expect( "Failed to start mailbox" );

		trace!( "calling mb.addr()" );
		let mut addr  = Addr::new( send.clone() );
		let mut addr2 = Addr::new( send.clone() );

		trace!( "calling addr.call( Ping( 'ping' ) )" );
		let result  = await!( addr.call( Ping( "ping".into() ) ) ).expect( "Call failed" );

		trace!( "calling addr.call( Ping( 'pang' ) )" );
		let result2 = await!( addr2.call( Ping( "pang".into() ) ) ).expect( "Call failed" );

		info!( "We got a result: {}", result );
		assert_eq!( "seedpingbla - after yield".to_string(), result );

		info!( "We got a result: {}", result2 );
		assert_eq!( "seedpingbla - after yieldpangbla - after yield".to_string(), result2 );
	};

	rt::spawn( program ).expect( "Spawn program" );

	rt::run();
}
