#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

#![ allow( dead_code, unused_imports )]


use
{
	thespis       :: { * } ,
	thespis_impl  :: { * } ,
	futures       :: { future::Future, task::{ LocalSpawn, SpawnExt }, executor::ThreadPool },
	std           :: { pin::Pin },
	log           :: { * },
};



struct MyActor
{
	seed: String,
}

impl MyActor
{
	async fn bla( x: &mut String ) -> String
	{
		x.extend( "bla".chars() );
		x.clone()
	}
}


impl Actor for MyActor {}


struct Ping(String);


impl Message for Ping
{
	type Result = String;
}


impl Handler< Ping > for MyActor
{
	fn handle( &mut self, msg: Ping ) -> Response<Ping> { Box::pin( async move
	{
		trace!( "Ping handler called" );

		self.seed.extend( msg.0.chars() );

		self.seed = await!( Self::bla( &mut self.seed ) );

		self.seed += " - after yield";

		self.seed.clone()

	})}
}



fn main()
{
	simple_logger::init().unwrap();

	let fut = async
	{
		let seed = "seed".into();
		let a = MyActor{ seed };

		trace!( "calling actor.start" );
		let mut mb  : ProcLocalMb<MyActor>   = a.start();

		trace!( "calling mb.addr()" );
		let mut addr: ProcLocalAddr<MyActor> = mb.addr();

		trace!( "calling addr.call( Ping(5) )" );
		let result = await!( addr.call( Ping( "ping".into() ) ) );

		info!( "We got a result: {}", result );
		assert_eq!( "seedpingbla - after yield".to_string(), result );

	};

	let _executor = ThreadPool::new().unwrap();

	futures::executor::block_on( fut );
}
