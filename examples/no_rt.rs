use
{
	futures       :: { task::{ LocalSpawnExt }, executor::LocalPool } ,
	thespis       :: { *                                            } ,
	thespis_impl  :: { *                                            } ,
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
	fn handle( &mut self, _msg: Ping ) -> Return<String> { Box::pin( async move
	{
		"pong".into()

	})}
}



fn main()
{
	let mut pool  = LocalPool::new();
	let mut exec  = pool.spawner();
	let mut exec2 = exec.clone();

	let program = async move
	{
		let actor = MyActor;

		// Create mailbox
		//
		let     mb   = Inbox::new()             ;
		let mut addr = Addr::new( mb.sender() ) ;

		// Now we can start and stop our mailbox whenever we want, because we have a future,
		// rather than thespis_impl spawning it for us.
		//
		exec2.spawn_local( mb.start_fut( actor ) ).expect( "Spawning mailbox failed" );

		let result  = addr.call( Ping( "ping".into() ) ).await.expect( "Call failed" );

		assert_eq!( "pong".to_string(), result );
		dbg!( result );
	};

	exec.spawn_local( program ).expect( "Spawn program" );

	pool.run();
}
