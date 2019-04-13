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
	let program = async move
	{
		let a = MyActor;

		// Create mailbox
		//
		let mb  : Inbox<MyActor> = Inbox::new();
		let mut addr             = Addr::new( mb.sender () );

		mb.start( a ).expect( "Failed to start mailbox" );

		thread::spawn( move ||
		{
			let thread_program = async move
			{
				let result  = await!( addr.call( Ping( "ping".into() ) ) ).expect( "Call failed" );

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
