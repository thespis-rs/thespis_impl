#![ feature( optin_builtin_traits ) ]
//
// Tested:
//
// - ✔ Spawn mailbox for actor that is !Send and !Sync
// - ✔ Spawn mailbox for actor that is Send and Sync but using the local methods
// - ✔ Manually spawn mailbox for actor that is !Send and !Sync
// - ✔ Manually spawn mailbox for actor that is Send and Sync but using the local methods
//
mod common;

use
{
	thespis         :: { *                                     } ,
	thespis_impl    :: { *,                                    } ,
	common          :: { actors::{ Sum, SumNoSend, Add, Show } } ,
	async_executors :: { LocalPool                             } ,
	futures         :: { task::LocalSpawnExt                   } ,
};




#[test]
//
fn test_not_send_actor()
{
	let mut exec  = LocalPool::default();
	let exec2 = exec.clone();

	let program = async move
	{

		// If we inline this in the next statement, it actually compiles with rt::spawn( program ) instead
		// of spawn_local.
		//
		let actor = SumNoSend(5);
		let mut addr = Addr::try_from_local( actor, &exec2 ).expect( "spawn actor mailbox" );

		addr.send( Add( 10 ) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );


		assert_eq!( 15, result );
	};

	exec.spawn_local( program ).expect( "spawn program" );
	exec.run();
}




#[test]
//
fn test_send_actor()
{
	let mut exec  = LocalPool::default();
	let exec2 = exec.clone();

	let program = async move
	{
		// If we inline this in the next statement, it actually compiles with rt::spawn( program ) instead
		// of spawn_local.
		//
		let actor = Sum(5);
		let mut addr = Addr::try_from_local( actor, &exec2 ).expect( "spawn actor mailbox" );

		addr.send( Add( 10 ) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 15, result );
	};

	exec.spawn_local( program ).expect( "spawn program" );
	exec.run();
}




#[test]
//
fn test_manually_not_send_actor()
{
	let mut exec  = LocalPool::default();
	let exec2 = exec.clone();

	let program = async move
	{
		// If we inline this in the next statement, it actually compiles with rt::spawn( program ) instead
		// of spawn_local.
		//
		let actor = SumNoSend(5);
		let mb    = Inbox::new( "SumNoSend".into() );

		let mut addr = Addr::new( mb.sender() );

		exec2.spawn_local( mb.start_fut_local( actor ) ).expect( "spawn actor mailbox" );

		addr.send( Add( 10 ) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 15, result );
	};

	exec.spawn_local( program ).expect( "spawn program" );
	exec.run();
}




#[test]
//
fn test_manually_send_actor()
{
	let mut exec  = LocalPool::default();
	let exec2 = exec.clone();

	let program = async move
	{
		// If we inline this in the next statement, it actually compiles with rt::spawn( program ) instead
		// of spawn_local.
		//
		let actor = Sum(5);
		let mb    = Inbox::new( "Sum".into() );

		let mut addr = Addr::new( mb.sender() );

		exec2.spawn_local( mb.start_fut_local( actor ) ).expect( "spawn actor mailbox" );

		addr.send( Add( 10 ) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 15, result );
	};

	exec.spawn_local( program ).expect( "spawn program" );
	exec.run();
}


