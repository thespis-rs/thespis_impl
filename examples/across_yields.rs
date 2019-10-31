use
{
	log               :: { *        } ,
	thespis           :: { *        } ,
	thespis_impl      :: { *        } ,
	async_executors   :: { *        } ,
	futures::executor :: { block_on } ,
};


#[ derive( Actor ) ]
//
struct MyActor { seed: String }

impl MyActor
{
	async fn bla( x: &mut String ) -> String
	{
		x.extend( "bla".chars() );
		x.clone()
	}
}


struct Ping( String );


impl Message for Ping
{
	type Return = String;
}


impl Handler< Ping > for MyActor
{
	// If you forget the move on the end, it won't compile and error messages will be shit!!!
	//
	fn handle( &mut self, msg: Ping ) -> Return<String> { Box::pin( async move
	{
		trace!( "Ping handler called" );

		self.seed.extend( msg.0.chars() );

		self.seed = Self::bla( &mut self.seed ).await;

		self.seed += " - after yield";

		self.seed.clone()

	})}
}



fn main()
{
	simple_logger::init().unwrap();

	let program = async move
	{
		let mut exec = ThreadPool::new().expect( "create threadpool" );
		let     a    = MyActor{ seed: "seed".into() }                          ;
		let mut addr = Addr::try_from( a, &mut exec ).expect( "Failed to create address" );

		let mut addr2 = addr.clone();

		trace!( "calling addr.call( Ping( 'ping' ) )" );
		let result  = addr.call( Ping( "ping".into() ) ).await.expect( "Call failed" );

		trace!( "calling addr.call( Ping( 'pang' ) )" );
		let result2 = addr2.call( Ping( "pang".into() ) ).await.expect( "Call failed" );

		info!( "We got a result: {}", result );
		assert_eq!( "seedpingbla - after yield".to_string(), result );

		info!( "We got a result: {}", result2 );
		assert_eq!( "seedpingbla - after yieldpangbla - after yield".to_string(), result2 );
	};

	block_on( program );
}
