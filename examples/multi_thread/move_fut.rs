#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

#![ allow( dead_code, unused_imports )]


use
{
	futures       :: { future::{ Future, FutureExt }, task::{ LocalSpawn, SpawnExt, LocalSpawnExt }, executor::LocalPool  } ,
	std           :: { pin::Pin, thread                                                                                   } ,
	log           :: { *                                                                                                  } ,
	thespis       :: { *                                                                                                  } ,
	thespis_impl  :: { multi_thread::*, runtime::rt                                                                       } ,
};



#[ derive( Actor ) ]
//
struct MyActor;

struct Ping( String );


impl Message for Ping
{
	type Return = String;
}


impl Handler< Ping > for MyActor
{
	fn handle( &mut self, _msg: Ping ) -> Return<String> { async move
	{
		"pong".into()

	}.boxed() }
}



fn main()
{
	let program = async move
	{
		let a = MyActor;

		// Create mailbox
		//
		let mb  : Inbox<MyActor> = Inbox::new();
		let mut addr             = Addr::new( mb.sender () );

		mb.start( a ).expect( "Failed to start mailbox" );

		// TODO: This might be a bug in async rust somewhere. It requires that addr is borrowed for static,
		// which makes no sense. Moving it into an async block here works, but it's an ugly workaround.
		//
		// let send_fut = addr.call( Ping( "ping".into() ) );
		//
		let send_fut = async move { await!( addr.call( Ping( "ping".into() ) ) ).expect( "Call failed" ) };

		thread::spawn( move ||
		{
			let thread_program = async move
			{
				let result = await!( send_fut );

				assert_eq!( "pong".to_string(), result );
				dbg!( result );
			};

			rt::spawn( thread_program ).expect( "Spawn thread 2 program" );

			rt::run();
		});
	};

	rt::spawn( program ).expect( "Spawn program" );

	rt::run();
}
