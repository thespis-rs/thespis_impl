use
{
	thespis           :: { *                    } ,
	thespis_impl      :: { *                    } ,
	futures::executor :: { block_on, ThreadPool } ,
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
	#[async_fn]	fn handle( &mut self, _msg: Ping ) -> String
	{
		"pong".into()
	}
}



fn main()
{
	block_on( async move
	{
		let     exec = ThreadPool::new().expect( "create threadpool" );
		let mut addr = Addr::try_from_actor( MyActor, &exec ).expect( "Failed to create address" );

		let result = addr.call( Ping( "ping".into() ) ).await.expect( "Call failed" );

		assert_eq!( "pong".to_string(), result );
		dbg!( result );

	});
}
