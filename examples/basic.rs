use
{
	thespis           :: { *        } ,
	thespis_impl      :: { *        } ,
	async_executors   :: { *        } ,
	futures::executor :: { block_on } ,
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
	block_on( async move
	{
		let     a    = MyActor                                                 ;
		let mut exec = ThreadPool::new().expect( "create threadpool" );
		let mut addr = Addr::try_from( a, &mut exec ).expect( "Failed to create address" );

		let result = addr.call( Ping( "ping".into() ) ).await.expect( "Call failed" );

		assert_eq!( "pong".to_string(), result );
		dbg!( result );

	});
}
