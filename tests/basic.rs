#![ feature( optin_builtin_traits ) ]

mod common;

use
{
	thespis         :: { *                          } ,
	thespis_impl    :: { *,                         } ,
	common          :: { actors::{ Sum, Add, Show } } ,
	async_executors :: { AsyncStd                   } ,
};




#[async_std::test]
//
async fn test_basic_send()
{
	let mut addr = Addr::try_from_actor( Sum(5), AsyncStd{} ).expect( "spawn actor mailbox" );

	addr.send( Add( 10 ) ).await.expect( "Send failed" );

	let result = addr.call( Show{} ).await.expect( "Call failed" );

	assert_eq!( 15, result );
}



#[async_std::test]
//
async fn test_basic_call()
{
	let mut addr = Addr::try_from_actor( Sum(5), AsyncStd{} ).expect( "spawn actor mailbox" );

	addr.call( Add(10) ).await.expect( "Send failed" );

	let result = addr.call( Show{} ).await.expect( "Call failed" );

	assert_eq!( 15, result );
}



#[async_std::test]
//
async fn send_from_multiple_addrs()
{
	let mut addr  = Addr::try_from_actor( Sum(5), AsyncStd{} ).expect( "spawn actor mailbox" );
	let mut addr2 = addr.clone();

	addr .send( Add( 10 ) ).await.expect( "Send failed" );
	addr2.send( Add( 10 ) ).await.expect( "Send failed" );

	let resp = addr.call( Show{} ).await.expect( "Call failed" );

	assert_eq!( 25, resp );
}



#[async_std::test]
//
async fn call_from_multiple_addrs()
{
	let mut addr  = Addr::try_from_actor( Sum(5), AsyncStd{} ).expect( "spawn actor mailbox" );
	let mut addr2 = addr.clone();

	addr .call( Add( 10 ) ).await.expect( "Send failed" );
	addr2.call( Add( 10 ) ).await.expect( "Send failed" );

	let resp = addr.call ( Show{} ).await.expect( "Call failed" );

	assert_eq!( 25, resp );
}
