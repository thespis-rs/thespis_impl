// Tested:
//
// - ✔ Send message to another thread
// - ✔ Call actor in another thread
// - ✔ Move the future from call to another thread and await it there
//
mod common;

use
{
	std             :: { thread                  } ,
	common          :: { *, actors::*, import::* } ,
	async_executors :: { AsyncStd                } ,
	futures         :: { executor::block_on      } ,
};



async fn move_addr_send() -> Result<u64, DynError >
{
	let mut addr = Addr::builder().start( Sum(5), &AsyncStd )?;
	let mut addr2            = addr.clone();

	thread::spawn( move ||
	{
		let thread_program = async move
		{
			addr2.send( Add( 10 ) ).await.expect( "Send failed" );
		};

		block_on( thread_program );

	}).join().expect( "join thread" );

	Ok(addr.call( Show{} ).await?)
}



async fn move_addr() -> Result<u64, DynError >
{
	let mut addr = Addr::builder().start( Sum(5), &AsyncStd )?;
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
	rx.await?;

	Ok( addr.call( Show{} ).await? )
}



async fn move_call() -> Result<u64, DynError >
{
	let mut addr  = Addr::builder().start( Sum(5), &AsyncStd )?;
	let mut addr2 = addr.clone();
	let (tx, rx)  = oneshot::channel::<()>();
	let call  = async move { addr2.call( Add( 10 ) ).await.expect( "call addr2" ) };

	thread::spawn( move ||
	{
		let thread_program = async move
		{
			call.await;
		};

		block_on( thread_program );

		tx.send(()).expect( "Signal end of thread" );

	});

	// TODO: create a way to join threads asynchronously...
	//
	rx.await?;

	Ok( addr.call( Show{} ).await? )
}


// Send message to another thread
//
#[async_std::test]
//
async fn test_basic_send() -> Result<(), DynError >
{
	// let _ = simple_logger::init();

	trace!( "start program" );

	let result = move_addr_send().await?;

	trace!( "result is: {}", result );
	assert_eq!( 15, result );

	Ok(())
}


// Call actor in another thread
//
#[async_std::test]
//
async fn test_basic_call() -> Result<(), DynError >
{
	// let _ = simple_logger::init();

	trace!( "start program" );

	let result = move_addr().await?;

	trace!( "result is: {}", result );
	assert_eq!( 15, result );

	Ok(())
}



// Move the future from call to another thread and await it there
//
#[async_std::test]
//
async fn test_move_call() -> Result<(), DynError >
{
	// let _ = simple_logger::init();

	trace!( "start program" );

	let result = move_call().await?;

	trace!( "result is: {}", result );
	assert_eq!( 15, result );

	Ok(())
}

