#![ feature( await_macro, async_await, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, optin_builtin_traits ) ]

mod common;

use
{
	thespis       :: { *                          } ,
	thespis_impl  :: { *, runtime::rt             } ,
	common        :: { actors::{ Sum, Add, Show } } ,
};




#[test]
//
fn test_basic_send()
{
	let program = async move
	{
		let mut addr = Addr::try_from( Sum(5) ).expect( "spawn actor mailbox" );

		await!( addr.send( Add( 10 ) ) ).expect( "Send failed" );

		let result = await!( addr.call( Show{}    ) ).expect( "Call failed" );

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

		await!( addr.call( Add( 10 ) ) ).expect( "Send failed" );

		let result = await!( addr.call( Show{}    ) ).expect( "Call failed" );

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

		await!( addr .send( Add( 10 ) ) ).expect( "Send failed" );
		await!( addr2.send( Add( 10 ) ) ).expect( "Send failed" );

		let resp = await!( addr.call ( Show{} ) ).expect( "Call failed" );

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

		await!( addr .call( Add( 10 ) ) ).expect( "Send failed" );
		await!( addr2.call( Add( 10 ) ) ).expect( "Send failed" );

		let resp = await!( addr.call ( Show{} ) ).expect( "Call failed" );

		assert_eq!( 25, resp );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}
