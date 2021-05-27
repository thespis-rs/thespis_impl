// Tested:
//
// - ✔ Spawn mailbox for actor that is !Send and !Sync
// - ✔ Spawn mailbox for actor that is Send and Sync but using the local methods
// - ✔ Manually spawn mailbox for actor that is !Send and !Sync
// - ✔ Manually spawn mailbox for actor that is Send and Sync but using the local methods
//
#![ cfg(not( target_arch = "wasm32" )) ]


mod common;

use
{
	common  :: { *, actors::*, import::*                  } ,
	futures :: { task::LocalSpawnExt, executor::LocalPool } ,
};



#[test]
//
fn test_not_send_actor() -> Result<(), DynError >
{
	let mut pool  = LocalPool::new();
	let     exec  = pool.spawner();
	let     exec2 = exec.clone();

	let program = async move
	{

		// If we inline this in the next statement, it actually compiles with rt::spawn( program ) instead
		// of spawn_local.
		//
		let mut addr = Addr::builder().start_local( SumNoSend::new(5), &exec2 ).expect( "start mailbox" );

		addr.send( Add( 10 ) ).await.expect( "Send" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );


		assert_eq!( 15, result );
	};

	exec.spawn_local( program )?;
	pool.run();

	Ok(())
}




#[test]
//
fn test_send_actor() -> Result<(), DynError >
{
	let mut pool  = LocalPool::new();
	let     exec  = pool.spawner();
	let     exec2 = exec.clone();

	let program = async move
	{
		// If we inline this in the next statement, it actually compiles with rt::spawn( program ) instead
		// of spawn_local.
		//
		let mut addr = Addr::builder().start_local( Sum(5), &exec2 ).expect( "spawn actor mailbox" );

		addr.send( Add( 10 ) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 15, result );
	};

	exec.spawn_local( program )?;
	pool.run();

	Ok(())
}



#[test]
//
fn test_manually_not_send_actor() -> Result<(), DynError >
{
	let mut pool  = LocalPool::new();
	let     exec  = pool.spawner();
	let     exec2 = exec.clone();

	let program = async move
	{
		let actor = SumNoSend::new(5);

		let (tx, rx) = mpsc::unbounded()                                         ;
		let mb       = Mailbox::new( Some("SumNoSend".into()), Box::new(rx) )    ;
		let tx       = Box::new(tx.sink_map_err( |e| Box::new(e) as SinkError )) ;
		let mut addr = mb.addr( tx )                                             ;

		exec2.spawn_local( async { mb.start_local( actor ).await; } ).expect( "spawn actor mailbox" );

		addr.send( Add( 10 ) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 15, result );
	};

	exec.spawn_local( program )?;
	pool.run();

	Ok(())
}




#[test]
//
fn test_manually_send_actor() -> Result<(), DynError >
{
	let mut pool  = LocalPool::new();
	let     exec  = pool.spawner();
	let     exec2 = exec.clone();

	let program = async move
	{
		// If we inline this in the next statement, it actually compiles with rt::spawn( program ) instead
		// of spawn_local.
		//
		let actor    = Sum(5)                                                    ;
		let (tx, rx) = mpsc::unbounded()                                         ;
		let mb       = Mailbox::new( Some("Sum".into()), Box::new(rx) )          ;
		let tx       = Box::new(tx.sink_map_err( |e| Box::new(e) as SinkError )) ;
		let mut addr = mb.addr( tx )                                             ;

		exec2.spawn_local( async { mb.start_local( actor ).await; } ).expect( "spawn actor mailbox" );

		addr.send( Add( 10 ) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 15, result );
	};

	exec.spawn_local( program )?;
	pool.run();

	Ok(())
}


