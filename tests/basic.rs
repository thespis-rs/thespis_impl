#![ feature( optin_builtin_traits ) ]

mod common;

use
{
	thespis         :: { *                          } ,
	thespis_impl    :: { *,                         } ,
	common          :: { actors::{ Sum, Add, Show } } ,
	async_executors :: { AsyncStd                   } ,
	futures         :: { executor::block_on         } ,
};




#[test]
//
fn test_basic_send()
{
	let program = async move
	{
		let mut exec = AsyncStd{};

		let mut addr = Addr::try_from( Sum(5), &mut exec ).expect( "spawn actor mailbox" );

		addr.send( Add( 10 ) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 15, result );
	};

	block_on( program );
}



#[test]
//
fn test_basic_call()
{
	let program = async move
	{
		let mut exec = AsyncStd{};

		let mut addr = Addr::try_from( Sum(5), &mut exec ).expect( "spawn actor mailbox" );

		addr.call( Add(10) ).await.expect( "Send failed" );

		let result = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 15, result );
	};

	block_on( program );
}



#[test]
//
fn send_from_multiple_addrs()
{
	let program = async move
	{
		let mut exec = AsyncStd{};

		let mut addr  = Addr::try_from( Sum(5), &mut exec ).expect( "spawn actor mailbox" );
		let mut addr2 = addr.clone();

		addr .send( Add( 10 ) ).await.expect( "Send failed" );
		addr2.send( Add( 10 ) ).await.expect( "Send failed" );

		let resp = addr.call( Show{} ).await.expect( "Call failed" );

		assert_eq!( 25, resp );
	};

	block_on( program );
}



#[test]
//
fn call_from_multiple_addrs()
{
	let program = async move
	{
		let mut exec = AsyncStd{};

		let sum = Sum(5);

		// Create mailbox
		//
		let     mb  : Inbox<Sum> = Inbox::new( Some( "Sum".into() ) ) ;
		let mut addr             = Addr ::new( mb.sender() )          ;
		let mut addr2            = addr.clone()                       ;

		mb.start( sum, &mut exec ).expect( "Failed to start mailbox" );

		addr .call( Add( 10 ) ).await.expect( "Send failed" );
		addr2.call( Add( 10 ) ).await.expect( "Send failed" );

		let resp = addr.call ( Show{} ).await.expect( "Call failed" );

		assert_eq!( 25, resp );
	};

	block_on( program );
}
