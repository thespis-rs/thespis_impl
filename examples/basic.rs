#![ feature( await_macro, async_await, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

#![ allow( dead_code, unused_imports )]


use
{
	futures       :: { future::{ Future, FutureExt }, task::{ LocalSpawn, SpawnExt, LocalSpawnExt }, executor::LocalPool  } ,
	std           :: { pin::Pin                      } ,
	log           :: { *                             } ,
	thespis       :: { *                             } ,
	thespis_impl  :: { *, runtime::rt } ,
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
	fn handle( &mut self, _msg: Ping ) -> ReturnNoSend<String> { Box::pin( async move
	{
		"pong".into()

	})}
}



fn main()
{
	rt::spawn( async move
	{
		let     a    = MyActor                                                 ;
		let mut addr = Addr::try_from( a ).expect( "Failed to create address" );

		let result = await!( addr.call( Ping( "ping".into() ) ) ).expect( "Call failed" );

		assert_eq!( "pong".to_string(), result );
		dbg!( result );

	}).expect( "Spawn program" );

	rt::run();
}
