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

	// call uses &mut self for Addr, so it's borrowed by the future. This means we can't just
	// move the future to another thread or spawn it directly. We have to move Addr with it.
	// This can be resolved by moving the Addr in an async block.
	//
	// let call_fut = addr.call( Ping( "ping".into() ) );
	//
	let call_fut = async move { addr.call( Ping( "ping".into() ) ).await };

	let handle = thread::spawn( move ||
	{
		block_on( async move
		{
			let result = call_fut.await.expect( "Call failed" );

			assert_eq!( "pong".to_string(), result );
			dbg!( result );
		});
	});

	handle.join().unwrap();

	Ok(())
}
