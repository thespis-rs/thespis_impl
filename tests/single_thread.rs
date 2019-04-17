#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, optin_builtin_traits ) ]

mod common;

use
{
	thespis       :: { *                             } ,
	thespis_impl  :: { single_thread::*, runtime::rt } ,
	common        :: { actors::{ Sum, Add, Show }    } ,
};

// We are testing single thread after all
//
impl !Send for Sum  {}
impl !Send for Add  {}
impl !Send for Show {}



async fn sum_send() -> u64
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new(             );
	let mut addr             = Addr ::new( mb.sender() );

	mb.start( sum ).expect( "Failed to start mailbox" );

	await!( addr.send( Add( 10 ) ) ).expect( "Send failed" );
	await!( addr.call( Show{}    ) ).expect( "Call failed" )
}



async fn sum_call() -> u64
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new(             );
	let mut addr             = Addr ::new( mb.sender() );

	mb.start( sum ).expect( "Failed to start mailbox" );

	await!( addr.call( Add( 10 ) ) ).expect( "Send failed" );
	await!( addr.call( Show{}    ) ).expect( "Call failed" )
}



#[test]
//
fn test_basic_send()
{
	let program = async move
	{
		assert_eq!( 15, await!( sum_send() ) );
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
		assert_eq!( 15, await!( sum_call() ) );
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
		let sum = Sum(5);

		// Create mailbox
		//
		let     mb  : Inbox<Sum> = Inbox::new(             );
		let mut addr             = Addr ::new( mb.sender() );
		let mut addr2            = addr.clone()             ;

		mb.start( sum ).expect( "Failed to start mailbox" );

		await!( addr.send ( Add( 10 ) ) ).expect( "Send failed" );
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

		await!( addr.call ( Add( 10 ) ) ).expect( "Send failed" );
		await!( addr2.call( Add( 10 ) ) ).expect( "Send failed" );

		let resp = await!( addr.call ( Show{} ) ).expect( "Call failed" );

		assert_eq!( 25, resp );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}
