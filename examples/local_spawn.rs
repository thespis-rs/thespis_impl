use
{
	thespis         :: { *                                         } ,
	thespis_impl    :: { Addr                                      } ,
	futures         :: { task::LocalSpawnExt, executor::LocalPool  } ,
	std             :: { marker::PhantomData, rc::Rc, error::Error } ,
};


#[ derive( Actor, Debug ) ] struct MyActor { i: u8, nosend: PhantomData<Rc<()>>}
#[ derive( Clone, Debug ) ] struct Ping( String )   ;


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
	#[async_fn_local] fn handle_local( &mut self, _msg: Ping ) -> String
	{
		// We can still access self across await points and mutably.
		//
		self.i = self.add( self.i ).await;
		dbg!( &self.i);
		"pong".into()
	}

	// It is necessary to provide handle in case of a !Send actor. It's a required method.
	// For Send actors that implement handle, handle_local is automatically provided.
	//
	#[async_fn] fn handle( &mut self, _msg: Ping ) -> String
	{
		unreachable!( "This actor is !Send and cannot be spawned on a threadpool" );
	}
}


#[async_std::main]
//
async fn main() -> Result< (), Box<dyn Error> >
{
	let mut pool = LocalPool::new();
	let     exec = pool.spawner();

	let actor    = MyActor { i: 3, nosend: PhantomData };
	let mut addr = Addr::builder().start_local( actor, &exec )?;

	exec.spawn_local( async move
	{
		let ping = Ping( "ping".into() );

		let result_local = addr.call( ping.clone() ).await.expect( "Call" );

		assert_eq!( "pong".to_string(), result_local );
		dbg!( result_local );

	})?;

	pool.run();

	Ok(())
}
