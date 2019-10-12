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
	thespis       :: { *                                     } ,
	thespis_impl  :: { *,                                    } ,
	async_runtime :: { rt, RtConfig                          } ,
	common        :: { actors::{ Sum, SumNoSend, Add, Show } } ,
};




#[test]
//
fn test_not_send_actor()
{
	// TODO: if we remove this, it panics, but cargo test exit's zero... we have to verify this.
	//
	rt::init( RtConfig::Local ).expect( "init async_runtime" );

	let program = async move
	{
		// If we inline this in the next statement, it actually compiles with rt::spawn( program ) instead
		// of spawn_local.
		//
		let actor = SumNoSend(5);
		let mut addr = Addr::try_from_local( actor ).expect( "spawn actor mailbox" );

		addr.send( Add( 10 ) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 15, result );
	};

	rt::spawn_local( program ).expect( "Spawn program" );
	rt::run();
}




#[test]
//
fn test_send_actor()
{
	// TODO: if we remove this, it panics, but cargo test exit's zero... we have to verify this.
	//
	rt::init( RtConfig::Local ).expect( "init async_runtime" );

	let program = async move
	{
		// If we inline this in the next statement, it actually compiles with rt::spawn( program ) instead
		// of spawn_local.
		//
		let actor = Sum(5);
		let mut addr = Addr::try_from_local( actor ).expect( "spawn actor mailbox" );

		addr.send( Add( 10 ) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 15, result );
	};

	rt::spawn_local( program ).expect( "Spawn program" );
	rt::run();
}




#[test]
//
fn test_manually_not_send_actor()
{
	// TODO: if we remove this, it panics, but cargo test exit's zero... we have to verify this.
	//
	rt::init( RtConfig::Local ).expect( "init async_runtime" );

	let program = async move
	{
		// If we inline this in the next statement, it actually compiles with rt::spawn( program ) instead
		// of spawn_local.
		//
		let actor = SumNoSend(5);
		let mb    = Inbox::new();

		let mut addr = Addr::new( mb.sender() );

		rt::spawn_local( mb.start_fut_local( actor ) ).expect( "spawn actor mailbox" );

		addr.send( Add( 10 ) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 15, result );
	};

	rt::spawn_local( program ).expect( "Spawn program" );
	rt::run();
}




#[test]
//
fn test_manually_send_actor()
{
	// TODO: if we remove this, it panics, but cargo test exit's zero... we have to verify this.
	//
	rt::init( RtConfig::Local ).expect( "init async_runtime" );

	let program = async move
	{
		// If we inline this in the next statement, it actually compiles with rt::spawn( program ) instead
		// of spawn_local.
		//
		let actor = Sum(5);
		let mb    = Inbox::new();

		let mut addr = Addr::new( mb.sender() );

		rt::spawn_local( mb.start_fut_local( actor ) ).expect( "spawn actor mailbox" );

		addr.send( Add( 10 ) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 15, result );
	};

	rt::spawn_local( program ).expect( "Spawn program" );
	rt::run();
}


