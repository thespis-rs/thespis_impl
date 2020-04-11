use
{
	std               :: { thread               } ,
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
	fn handle( &mut self, _msg: Ping ) -> Return<String> { Box::pin( async move
	{
		"pong".into()

	})}
}



fn main()
{
	let program = async move
	{
		let     exec = ThreadPool::new().expect( "create threadpool" );
		let mut addr = Addr::try_from_actor( MyActor, &exec ).expect( "Failed to create address" );

		// TODO: This might be a bug in async rust somewhere. It requires that addr is borrowed for static,
		// which makes no sense. Moving it into an async block here works, but it's an ugly workaround.
		//
		// let call_fut = addr.call( Ping( "ping".into() ) );
		//
		let call_fut = async move { addr.call( Ping( "ping".into() ) ).await.expect( "Call failed" ) };

		thread::spawn( move ||
		{
			let thread_program = async move
			{
				let result = call_fut.await;

				assert_eq!( "pong".to_string(), result );
				dbg!( result );
			};

			block_on( thread_program );
		});
	};

	block_on( program );
}
