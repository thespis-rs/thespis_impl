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


#[ derive( Actor ) ] struct MyActor;
#[ derive( Actor ) ] struct Other  ;

struct Ping( String );


impl Message for Ping
{
	type Result = String;
}


impl Handler< Ping > for MyActor
{
	fn handle( &mut self, _msg: Ping ) -> Return<String> { async move
	{
		"MyActor".into()

	}.boxed() }
}


impl Handler< Ping > for Other
{
	fn handle( &mut self, _msg: Ping ) -> Return<String> { async move
	{
		"Other".into()

	}.boxed() }
}



fn main()
{
	let program = async move
	{
		let a = MyActor;
		let b = Other;

		// Create mailbox
		//
		let mb  : Inbox<MyActor> = Inbox::new();
		let addr                 = Addr::new( mb.sender() );

		// Create Other mailbox
		//
		let mbo  : Inbox<Other> = Inbox::new();
		let addro               = Addr::new( mbo.sender() );

		mb .start( a ).expect( "Failed to start mailbox" );
		mbo.start( b ).expect( "Failed to start mailbox" );

		let recs = vec![ addr.recipient(), addro.recipient() ];

		for mut actor in recs
		{
			println!( "Pinged: {}", await!( actor.call( Ping( "ping".into() ) ) ).expect( "Call failed" ) );
		}
	};

	rt::spawn( program ).expect( "Spawn program" );

	rt::run();
}
