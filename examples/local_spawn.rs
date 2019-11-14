#![ feature( optin_builtin_traits ) ]
//
use
{
	thespis         :: { Actor, Message, Handler, Return, ReturnNoSend, Recipient } ,
	thespis_impl    :: { Addr                                                     } ,
	async_executors :: { *                                                        } ,
	futures         :: { task::LocalSpawnExt                                      } ,
};


#[ derive( Actor, Debug ) ] struct MyActor { i: u8 }
#[ derive( Clone, Debug ) ] struct Ping( String )   ;

impl !Send for MyActor {}

impl Message for Ping
{
	type Return = String;
}



impl MyActor
{
	async fn add( &mut self, mut x: u8 ) -> u8
	{
		x += 1;
		x
	}
}



impl Handler<Ping> for MyActor
{
	// Implement handle_local to enable !Send actor an mailbox.
	//
	fn handle_local( &mut self, _msg: Ping ) -> ReturnNoSend<String> { Box::pin( async move
	{
		// We can still access self across await points and mutably.
		//
		self.i = self.add( self.i ).await;
		dbg!( &self.i);
		"pong".into()

	})}

	// It is necessary to provide handle in case of a !Send actor. It's a required method.
	// For Send actors that implement handle, handle_local is automatically provided.
	//
	fn handle( &mut self, _msg: Ping ) -> Return<String>
	{
		unreachable!( "This actor is !Send and cannot be spawned on a threadpool" );
	}
}


fn main()
{
	let mut exec = LocalPool::default();

	let actor    = MyActor { i: 3 };
	let mut addr = Addr::try_from_local( actor, &exec ).expect( "spawn actor locally" );

	exec.spawn_local( async move
	{
		let ping = Ping( "ping".into() );

		let result_local  = addr.call( ping.clone() ).await.expect( "Call local"  );

		assert_eq!( "pong".to_string(), result_local );
		dbg!( result_local );

	}).expect( "spawn local" );

	exec.run();
}
