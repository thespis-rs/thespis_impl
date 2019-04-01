#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

#![ allow( dead_code, unused_imports )]


use
{
	futures       :: { future::{ Future, FutureExt }, task::{ LocalSpawn, Spawn, SpawnExt, LocalSpawnExt }, executor::LocalPool, channel::oneshot } ,
	thespis       :: { * } ,
	log           :: { * } ,
	thespis_impl  :: { multi_thread::* } ,
	std           :: { pin::Pin, thread } ,
};


#[ derive( Actor ) ]
//
struct Sum( u64 );

#[ derive( Debug ) ] struct Add( u64 );
#[ derive( Debug ) ] struct Show;

impl Message for Add  { type Result = ();  }
impl Message for Show { type Result = u64; }



impl Handler< Add > for Sum
{
	fn handle( &mut self, msg: Add ) -> Response<()> { async move
	{
		trace!( "called sum with: {:?}", msg );

		self.0 += msg.0;

	}.boxed() }
}



impl Handler< Show > for Sum
{
	fn handle( &mut self, _msg: Show ) -> Response<u64> { async move
	{
		trace!( "called sum with: Show" );

		self.0

	}.boxed() }
}



impl Drop for Sum
{
	fn drop( &mut self )
	{
		trace!( "dropping Sum")
	}
}



async fn sum_send( exec: &mut impl LocalSpawn ) -> u64
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new(             );
	let mut addr             = Addr ::new( mb.sender() );
	let mut addr2            = addr.clone();

	// This is ugly right now. It will be more ergonomic in the future.
	//
	let move_mb = async move { await!( mb.start( sum ) ); };
	exec.spawn_local( move_mb ).expect( "Spawning mailbox failed" );

	thread::spawn( move ||
	{
		let mut thread_pool  = LocalPool::new();
		let mut thread_exec  = thread_pool.spawner();
		let mut thread_exec2 = thread_exec.clone();

		let thread_program = async move
		{
			// This is ugly right now. It will be more ergonomic in the future.
			//
			let move_addr = async move { await!( addr2.send( Add( 10 ) ) ); };
			thread_exec2.spawn_local( move_addr ).expect( "Spawning mailbox failed" );
		};

		thread_exec.spawn_local( thread_program ).expect( "Spawn thread_program" );

		thread_pool.run();

	}).join().expect( "join thread" );

	await!( addr.call( Show{} ) )
}



async fn sum_call( exec: &mut impl LocalSpawn ) -> u64
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new(             );
	let mut addr             = Addr ::new( mb.sender() );
	let mut addr2            = addr.clone();

	// This is ugly right now. It will be more ergonomic in the future.
	//
	let move_mb = async move { await!( mb.start( sum ) ); };
	exec.spawn_local( move_mb ).expect( "Spawning mailbox failed" );

	let (tx, rx) = oneshot::channel::<()>();


	thread::spawn( move ||
	{
		let mut thread_pool  = LocalPool::new();
		let mut thread_exec  = thread_pool.spawner();
		let mut thread_exec2 = thread_exec.clone();

		let thread_program = async move
		{
			// This is ugly right now. It will be more ergonomic in the future.
			//
			let move_addr = async move { await!( addr2.call( Add( 10 ) ) ); };
			thread_exec2.spawn_local( move_addr ).expect( "Spawning mailbox failed" );
		};

		thread_exec.spawn_local( thread_program ).expect( "Spawn thread_program" );

		thread_pool.run();

		tx.send(()).expect( "Signal end of thread" );

	});

	await!( rx ).expect( "receive Signal end of thread" );

	await!( addr.call( Show{} ) )
}



#[test]
//
fn test_basic_send()
{
	let mut pool  = LocalPool::new();
	let mut exec  = pool.spawner();
	let mut exec2 = exec.clone();

	let program = async move
	{
		let _ = simple_logger::init();

		trace!( "start program" );

		let result = await!( sum_send( &mut exec2 ) );

		trace!( "result is: {}", result );
		assert_eq!( 15, result );
	};

	exec.spawn_local( program ).expect( "failed to spawn test" );

	pool.run();
}



#[test]
//
fn test_basic_call()
{
	let mut pool  = LocalPool::new();
	let mut exec  = pool.spawner();
	let mut exec2 = exec.clone();

	let program = async move
	{
		let _ = simple_logger::init();

		trace!( "start program" );

		let result = await!( sum_call( &mut exec2 ) );

		trace!( "result is: {}", result );
		assert_eq!( 15, result );
	};

	exec.spawn_local( program ).expect( "failed to spawn test" );

	pool.run();
}

