#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, optin_builtin_traits ) ]

#![ allow( dead_code, unused_imports )]

mod common;

use
{
	futures       :: { future::{ Future, FutureExt }, task::{ LocalSpawn, Spawn, SpawnExt, LocalSpawnExt }, executor::LocalPool } ,
	thespis       :: { * } ,
	log           :: { * } ,
	thespis_impl  :: { single_thread::* } ,
	std           :: { pin::Pin         } ,
	common        :: actors::{ Sum, Add, Show   } ,
};

// We are testing single thread after all
//
impl !Send for Sum  {}
impl !Send for Add  {}
impl !Send for Show {}



async fn sum_send( exec: &mut impl LocalSpawn ) -> u64
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new(             );
	let mut addr             = Addr ::new( mb.sender() );

	// This is ugly right now. It will be more ergonomic in the future.
	//
	let move_mb = async move { await!( mb.start_fut( sum ) ); };
	exec.spawn_local( move_mb ).expect( "Spawning mailbox failed" );


	await!( addr.send( Add( 10 ) ) );

	let res = await!( addr.call( Show{} ) );

	trace!( "res is: {}", res );

	res
}



async fn sum_call( exec: &mut impl LocalSpawn ) -> u64
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new(             );
	let mut addr             = Addr ::new( mb.sender() );

	// This is ugly right now. It will be more ergonomic in the future.
	//
	let move_mb = async move { await!( mb.start_fut( sum ) ); };
	exec.spawn_local( move_mb ).expect( "Spawning mailbox failed" );


	await!( addr.call( Add( 10 ) ) );

	let res = await!( addr.call( Show{} ) );

	trace!( "res is: {}", res );

	res
}



#[test]
//
fn test_basic_send()
{
	let mut pool = LocalPool::new();
	let mut exec = pool.spawner();

	let program = async move
	{
		let _ = simple_logger::init();

		let result = await!( sum_send( &mut exec ) );

		trace!( "result is: {}", result );
		assert_eq!( 15, result );
	};

	pool.run_until( program );
}



#[test]
//
fn test_basic_call()
{
	let mut pool = LocalPool::new();
	let mut exec = pool.spawner();

	let program = async move
	{
		let _ = simple_logger::init();

		let result = await!( sum_call( &mut exec ) );

		trace!( "result is: {}", result );
		assert_eq!( 15, result );
	};

	pool.run_until( program );
}
