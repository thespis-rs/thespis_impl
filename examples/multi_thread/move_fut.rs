#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

#![ allow( dead_code, unused_imports )]


use
{
	futures       :: { future::{ Future, FutureExt }, task::{ LocalSpawn, SpawnExt, LocalSpawnExt }, executor::LocalPool  } ,
	std           :: { pin::Pin, thread                                                                                   } ,
	log           :: { *                                                                                                  } ,
	thespis       :: { *                                                                                                  } ,
	thespis_impl  :: { multi_thread::*                                                                                    } ,
};



#[ derive( Actor ) ]
//
struct MyActor;

struct Ping( String );


impl Message for Ping
{
	type Result = String;
}


impl Handler< Ping > for MyActor
{
	fn handle( &mut self, _msg: Ping ) -> Response<String> { async move
	{
		"pong".into()

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

		// Create mailbox
		//
		let mb  : Inbox<MyActor> = Inbox::new();
		let mut addr             = Addr::new( mb.sender () );

		// This is ugly right now. It will be more ergonomic in the future.
		//
		let move_mb = async move { await!( mb.start( a ) ); };
		exec2.spawn_local( move_mb ).expect( "Spawning mailbox failed" );

		// This might be a bug in async rust somewhere. It requires that addr is borrowed for static,
		// which makes no sense. Moving it into an async block here works, but it's an ugly workaround.
		//
		// let send_fut = addr.call( Ping( "ping".into() ) );
		//
		let send_fut = async move { await!( addr.call( Ping( "ping".into() ) ) ) };

		thread::spawn( move ||
		{
			let mut thread_pool = LocalPool::new();

			let thread_program = async move
			{
				let result = await!( send_fut );

				assert_eq!( "pong".to_string(), result );
				dbg!( result );
			};

			thread_pool.run_until( thread_program );
		});
	};

	exec.spawn_local( program ).expect( "Spawn program" );

	pool.run();
}
