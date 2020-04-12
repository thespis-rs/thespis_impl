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
	#[async_fn] fn handle( &mut self, _msg: Ping ) -> String
	{
		"pong".into()
	}
}



#[async_std::main]
//
async fn main()
{
	let     exec = ThreadPool::new().expect( "create threadpool" );
	let mut addr = Addr::try_from_actor( MyActor, &exec ).expect( "Failed to create address" );

	let handle = thread::spawn( move ||
	{
		let thread_program = async move
		{
			let result  = addr.call( Ping( "ping".into() ) ).await.expect( "Call failed" );

			assert_eq!( "pong".to_string(), result );
			dbg!( result );
		};

		block_on( thread_program );
	});

	handle.join().expect( "thread succeeds" );
}
