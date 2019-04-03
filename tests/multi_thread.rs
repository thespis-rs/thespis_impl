#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

#![ allow( dead_code, unused_imports )]

mod common;

use
{
	futures       :: { future::{ Future, FutureExt }, task::{ LocalSpawn, Spawn, SpawnExt, LocalSpawnExt } } ,
	futures       :: { executor::LocalPool, channel::oneshot                                               } ,
	thespis       :: { * } ,
	log           :: { * } ,
	thespis_impl  :: { multi_thread::*, runtime::rt } ,
	std           :: { pin::Pin, thread } ,
	common        :: actors::{ Sum, Add, Show   } ,
};


async fn sum_send() -> u64
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new(             );
	let mut addr             = Addr ::new( mb.sender() );
	let mut addr2            = addr.clone();

	mb.start( sum ).expect( "Failed to start mailbox" );

	thread::spawn( move ||
	{
		let thread_program = async move
		{
			await!( addr2.send( Add( 10 ) ) );
		};

		rt::spawn( thread_program ).expect( "Spawn thread 2 program" );
		rt::run();

	}).join().expect( "join thread" );

	await!( addr.call( Show{} ) )
}



async fn sum_call() -> u64
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new(             );
	let mut addr             = Addr ::new( mb.sender() );
	let mut addr2            = addr.clone();

	mb.start( sum ).expect( "Failed to start mailbox" );


	let (tx, rx) = oneshot::channel::<()>();


	thread::spawn( move ||
	{
		let thread_program = async move
		{
			await!( addr2.call( Add( 10 ) ) );
		};

		rt::spawn( thread_program ).expect( "Spawn thread 2 program" );
		rt::run();

		tx.send(()).expect( "Signal end of thread" );

	});

	// TODO: create a way to join threads asynchronously...
	//
	await!( rx ).expect( "receive Signal end of thread" );

	await!( addr.call( Show{} ) )
}



#[test]
//
fn test_basic_send()
{
	let program = async move
	{
		let _ = simple_logger::init();

		trace!( "start program" );

		let result = await!( sum_send() );

		trace!( "result is: {}", result );
		assert_eq!( 15, result );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}



#[test]
//
fn test_basic_call()
{
	let program = async move
	{
		let _ = simple_logger::init();

		trace!( "start program" );

		let result = await!( sum_call() );

		trace!( "result is: {}", result );
		assert_eq!( 15, result );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}

