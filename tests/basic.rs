#![ feature( async_await, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, optin_builtin_traits ) ]

mod common;

use
{
	thespis       :: { *                          } ,
	thespis_impl  :: { *,                         } ,
	async_runtime :: { rt                         } ,
	common        :: { actors::{ Sum, Add, Show } } ,
};




#[test]
//
fn test_basic_send()
{
	let program = async move
	{
		let mut addr = Addr::try_from( Sum(5) ).expect( "spawn actor mailbox" );

		addr.send( Add( 10 ) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );

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
		let mut addr = Addr::try_from( Sum(5) ).expect( "spawn actor mailbox" );

		addr.call( Add(10) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 15, result );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}



#[test]
//
fn send_from_multiple_addrs()
{
	let program = async move
	{
		let mut addr  = Addr::try_from( Sum(5) ).expect( "spawn actor mailbox" );
		let mut addr2 = addr.clone();

		addr .send( Add( 10 ) ).await.expect( "Send failed" );
		addr2.send( Add( 10 ) ).await.expect( "Send failed" );

		let resp = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 25, resp );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}



#[test]
//
fn call_from_multiple_addrs()
{
	let program = async move
	{
		let sum = Sum(5);

		// Create mailbox
		//
		let     mb  : Inbox<Sum> = Inbox::new(             );
		let mut addr             = Addr ::new( mb.sender() );
		let mut addr2            = addr.clone()             ;

		mb.start( sum ).expect( "Failed to start mailbox" );

		addr .call( Add( 10 ) ).await.expect( "Send failed" );
		addr2.call( Add( 10 ) ).await.expect( "Send failed" );

		let resp = addr.call ( Show{} ).await.expect( "Call failed" );

		assert_eq!( 25, resp );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}
