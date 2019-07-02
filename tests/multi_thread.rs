#![ feature( async_await, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, optin_builtin_traits ) ]

// Tested:
//
// - ✔ Send message to another thread
// - ✔ Call actor in another thread
// - ✔ Move the future from call to another thread and await it there

mod common;

use
{
	futures       :: { channel::oneshot           } ,
	thespis       :: { *                          } ,
	log           :: { *                          } ,
	thespis_impl  :: { *                          } ,
	async_runtime :: { rt                         } ,
	std           :: { thread                     } ,
	common        :: { actors::{ Sum, Add, Show } } ,
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
			addr2.send( Add( 10 ) ).await.expect( "Send failed" );
		};

		rt::spawn( thread_program ).expect( "Spawn thread 2 program" );
		rt::run();

	}).join().expect( "join thread" );

	addr.call( Show{} ).await.expect( "Call failed" )
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
			addr2.call( Add( 10 ) ).await.expect( "Call failed" );
		};

		rt::spawn( thread_program ).expect( "Spawn thread 2 program" );
		rt::run();

		tx.send(()).expect( "Signal end of thread" );

	});

	// TODO: create a way to join threads asynchronously...
	//
	rx.await.expect( "receive Signal end of thread" );

	addr.call( Show{} ).await.expect( "Call failed" )
}



async fn move_call() -> u64
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new(             );
	let mut addr             = Addr ::new( mb.sender() );
	let mut addr2            = addr.clone();

	mb.start( sum ).expect( "Failed to start mailbox" );


	let (tx, rx) = oneshot::channel::<()>();
	let call_fut = async move { addr2.call( Add( 10 ) ).await.expect( "Call failed" ) };

	thread::spawn( move ||
	{
		let thread_program = async move
		{
			call_fut.await;
		};

		rt::spawn( thread_program ).expect( "Spawn thread 2 program" );
		rt::run();

		tx.send(()).expect( "Signal end of thread" );

	});

	// TODO: create a way to join threads asynchronously...
	//
	rx.await.expect( "receive Signal end of thread" );

	addr.call( Show{} ).await.expect( "Call failed" )
}


// Send message to another thread
//
#[test]
//
fn test_basic_send()
{
	let program = async move
	{
		// let _ = simple_logger::init();

		trace!( "start program" );

		let result = sum_send().await;

		trace!( "result is: {}", result );
		assert_eq!( 15, result );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}


// Call actor in another thread
//
#[test]
//
fn test_basic_call()
{
	let program = async move
	{
		// let _ = simple_logger::init();

		trace!( "start program" );

		let result = sum_call().await;

		trace!( "result is: {}", result );
		assert_eq!( 15, result );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}



// Move the future from call to another thread and await it there
//
#[test]
//
fn test_move_call()
{
	let program = async move
	{
		// let _ = simple_logger::init();

		trace!( "start program" );

		let result = move_call().await;

		trace!( "result is: {}", result );
		assert_eq!( 15, result );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}

