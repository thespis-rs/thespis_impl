use
{
	std               :: { thread               } ,
	thespis           :: { *                    } ,
	thespis_impl      :: { *                    } ,
	futures::executor :: { block_on, ThreadPool } ,
	std               :: { error::Error         } ,
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
async fn main() -> Result< (), Box<dyn Error> >
{
	let     exec = ThreadPool::new()?;
	let mut addr = Addr::builder().start( MyActor, &exec )?;

	let handle = thread::spawn( move ||
	{
		let thread_program = async move
		{
			let result  = addr.call( Ping( "ping".into() ) ).await.expect( "Call" );

			assert_eq!( "pong".to_string(), result );
			dbg!( result );
		};

		block_on( thread_program );
	});

	handle.join().unwrap();

	Ok(())
}
