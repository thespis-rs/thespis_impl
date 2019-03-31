#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

#![ allow( dead_code, unused_imports )]


use
{
	futures       :: { future::{ Future, FutureExt }, task::{ LocalSpawn, SpawnExt, LocalSpawnExt }, executor::LocalPool  } ,
	std           :: { pin::Pin                                                                            } ,
	log           :: { *                                                                                   } ,
	thespis       :: { *                                                                                   } ,
	thespis_impl  :: { single_thread::*                                                                    } ,
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
	fn handle( &mut self, _msg: Ping ) -> Response<Ping> { async move
	{
		"MyActor".into()

	}.boxed() }
}


impl Handler< Ping > for Other
{
	fn handle( &mut self, _msg: Ping ) -> Response<Ping> { async move
	{
		"Other".into()

	}.boxed() }
}



fn main()
{
	let mut pool  = LocalPool::new();
	let mut exec  = pool.spawner();
	let mut exec2 = exec.clone();

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

		// This is ugly right now. It will be more ergonomic in the future.
		//
		let move_mb = async move { await!( mb.start( a ) ); };
		exec2.spawn_local( move_mb ).expect( "Spawning mailbox failed" );

		// This is ugly right now. It will be more ergonomic in the future.
		//
		let move_mbo = async move { await!( mbo.start( b ) ); };
		exec2.spawn_local( move_mbo ).expect( "Spawning mailbox failed" );

		let recs = vec![ addr.recipient(), addro.recipient() ];

		for mut actor in recs
		{
			println!( "Pinged: {}", await!( actor.call( Ping( "ping".into() ) ) ) );
		}

	};

	exec.spawn_local( program ).expect( "Spawn program" );

	pool.run();
}
