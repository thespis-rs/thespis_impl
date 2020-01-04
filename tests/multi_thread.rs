#![ feature( optin_builtin_traits ) ]

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
	std           :: { thread                     } ,
	common        :: { actors::{ Sum, Add, Show } } ,
	async_executors :: { AsyncStd                 } ,
	futures         :: { executor::block_on       } ,
};



async fn sum_send() -> u64
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new( Some( "Sum".into() ) );
	let mut addr             = Addr ::new( mb.sender() );
	let mut addr2            = addr.clone();
	let mut exec             = AsyncStd{};

	mb.start( sum, &mut exec ).expect( "Failed to start mailbox" );

	thread::spawn( move ||
	{
		let thread_program = async move
		{
			addr2.send( Add( 10 ) ).await.expect( "Send failed" );
		};

		block_on( thread_program );

	}).join().expect( "join thread" );

	addr.call( Show{} ).await.expect( "Call failed" )
}



async fn sum_call() -> u64
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new( Some( "Sum".into() ) );
	let mut addr             = Addr ::new( mb.sender() );
	let mut addr2            = addr.clone();
	let mut exec             = AsyncStd{};

	mb.start( sum, &mut exec ).expect( "Failed to start mailbox" );


	let (tx, rx) = oneshot::channel::<()>();


	thread::spawn( move ||
	{
		let thread_program = async move
		{
			addr2.call( Add( 10 ) ).await.expect( "Call failed" );
		};

		block_on( thread_program );

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
	let     mb  : Inbox<Sum> = Inbox::new( Some( "Sum".into() ) );
	let mut addr             = Addr ::new( mb.sender() );
	let mut addr2            = addr.clone();
	let mut exec             = AsyncStd{};

	mb.start( sum, &mut exec ).expect( "Failed to start mailbox" );


	let (tx, rx) = oneshot::channel::<()>();
	let call_fut = async move { addr2.call( Add( 10 ) ).await.expect( "Call failed" ) };

	thread::spawn( move ||
	{
		let thread_program = async move
		{
			call_fut.await;
		};

		block_on( thread_program );

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

	block_on( program );
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

	block_on( program );
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

	block_on( program );
}

