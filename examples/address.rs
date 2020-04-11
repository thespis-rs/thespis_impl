use
{
	thespis           :: { *                    } ,
	thespis_impl      :: { *                    } ,
	futures::executor :: { block_on, ThreadPool } ,
};


#[ derive( Actor ) ] struct MyActor;
#[ derive( Actor ) ] struct Other  ;

struct Ping( String );


impl Message for Ping
{
	type Return = String;
}


impl Handler< Ping > for MyActor
{
	fn handle( &mut self, _msg: Ping ) -> Return<String> { Box::pin( async move
	{
		"MyActor".into()

	})}
}


impl Handler< Ping > for Other
{
	fn handle( &mut self, _msg: Ping ) -> Return<String> { Box::pin( async move
	{
		"Other".into()

	})}
}



fn main()
{
	let program = async move
	{
		let exec  = ThreadPool::new().expect( "create threadpool" );
		let addr  = Addr::try_from_actor( MyActor, &exec ).expect( "Failed to create address" );
		let addro = Addr::try_from_actor( Other  , &exec ).expect( "Failed to create address" );


		let recs: Vec< BoxAddress<Ping, ThesErr> > = vec![ Box::new( addr ), Box::new( addro ) ];

		// or like this, but it clones internally. Note that the compiler is capable here of detecting
		// that we want a Address to the message type Ping.
		//
		// let recs = vec![ addr.recipient(), addro.recipient() ];
		//
		// In a more complex situation, it might be necessary to annotate the type.

		for mut actor in recs
		{
			println!( "Pinged: {}", actor.call( Ping( "ping".into() ) ).await.expect( "Call failed" ) );
		}
	};

	block_on( program );
}
