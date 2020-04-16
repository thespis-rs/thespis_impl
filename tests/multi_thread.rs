// Tested:
//
// - ✔ Send message to another thread
// - ✔ Call actor in another thread
// - ✔ Move the future from call to another thread and await it there

mod common;

use
{
	futures         :: { channel::oneshot           } ,
	thespis         :: { *                          } ,
	log             :: { *                          } ,
	thespis_impl    :: { *                          } ,
	std             :: { thread                     } ,
	common          :: { actors::{ Sum, Add, Show } } ,
	async_executors :: { AsyncStd                   } ,
	futures         :: { executor::block_on         } ,
};



async fn move_addr_send() -> u64
{
	let mut addr = Addr::builder().start( Sum(5), &AsyncStd ).expect( "spawn actor mailbox" );
	let mut addr2            = addr.clone();

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



async fn move_addr() -> u64
{
	let mut addr = Addr::builder().start( Sum(5), &AsyncStd ).expect( "spawn actor mailbox" );
	let mut addr2            = addr.clone();

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
	let mut addr  = Addr::builder().start( Sum(5), &AsyncStd ).expect( "spawn actor mailbox" );
	let mut addr2 = addr.clone();
	let (tx, rx)  = oneshot::channel::<()>();
	let call_fut  = async move { addr2.call( Add( 10 ) ).await.expect( "Call failed" ) };

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
#[async_std::test]
//
async fn test_basic_send()
{
	// let _ = simple_logger::init();

	trace!( "start program" );

	let result = move_addr_send().await;

	trace!( "result is: {}", result );
	assert_eq!( 15, result );
}


// Call actor in another thread
//
#[async_std::test]
//
async fn test_basic_call()
{
	// let _ = simple_logger::init();

	trace!( "start program" );

	let result = move_addr().await;

	trace!( "result is: {}", result );
	assert_eq!( 15, result );
}



// Move the future from call to another thread and await it there
//
#[async_std::test]
//
async fn test_move_call()
{
	// let _ = simple_logger::init();

	trace!( "start program" );

	let result = move_call().await;

	trace!( "result is: {}", result );
	assert_eq!( 15, result );
}

