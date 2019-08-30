#![ allow( unused_imports, dead_code ) ]
#![ feature( arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, optin_builtin_traits ) ]

mod common;

use
{
	futures       :: { future::{ Future, FutureExt }, task::{ LocalSpawn, Spawn, SpawnExt, LocalSpawnExt } } ,
	futures       :: { executor::LocalPool, channel::oneshot                                               } ,
	thespis       :: { * } ,
	log           :: { * } ,
	thespis_impl  :: { *  } ,
	std           :: { pin::Pin, thread } ,
	common        :: actors::{ Sum, Add, Show   } ,
};


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
	let move_mb = async move { mb.start_fut( sum ).await; };
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
			let move_addr = async move { addr2.send( Add( 10 ) ).await.expect( "Call failed" ); };
			thread_exec2.spawn_local( move_addr ).expect( "Spawning mailbox failed" );
		};

		thread_exec.spawn_local( thread_program ).expect( "Spawn thread_program" );

		thread_pool.run();

	}).join().expect( "join thread" );

	addr.call( Show{} ).await.expect( "Call failed" )
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
	let move_mb = async move { mb.start_fut( sum ).await; };
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
			let move_addr = async move { addr2.call( Add( 10 ) ).await.expect( "Call failed" ); };
			thread_exec2.spawn_local( move_addr ).expect( "Spawning mailbox failed" );
		};

		thread_exec.spawn_local( thread_program ).expect( "Spawn thread_program" );

		thread_pool.run();

		tx.send(()).expect( "Signal end of thread" );

	});

	rx.await.expect( "receive Signal end of thread" );

	addr.call( Show{} ).await.expect( "Call failed" )
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
		// let _ = simple_logger::init();

		trace!( "start program" );

		let result = sum_send( &mut exec2 ).await;

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
		// let _ = simple_logger::init();

		trace!( "start program" );

		let result = sum_call( &mut exec2 ).await;

		trace!( "result is: {}", result );
		assert_eq!( 15, result );
	};

	exec.spawn_local( program ).expect( "failed to spawn test" );

	pool.run();
}

