#![ feature( optin_builtin_traits ) ]

mod common;

use
{
	thespis         :: { *                                     } ,
	thespis_impl    :: { *,                                    } ,
	common          :: { actors::{ Sum, Add, Show }, import::* } ,
	async_executors :: { AsyncStd                              } ,
};




#[async_std::test]
//
async fn test_basic_send()
{
	let mut exec = AsyncStd{};

	let mut addr = Addr::try_from( Sum(5), &mut exec ).expect( "spawn actor mailbox" );

	addr.send( Add( 10 ) ).await.expect( "Send failed" );

	let result = addr.call( Show{} ).await.expect( "Call failed" );

	assert_eq!( 15, result );
}



#[async_std::test]
//
async fn test_basic_call()
{
	let mut exec = AsyncStd{};

	let mut addr = Addr::try_from( Sum(5), &mut exec ).expect( "spawn actor mailbox" );

	addr.call( Add(10) ).await.expect( "Send failed" );

	let result = addr.call( Show{} ).await.expect( "Call failed" );

	assert_eq!( 15, result );
}



#[async_std::test]
//
async fn send_from_multiple_addrs()
{
	let mut exec = AsyncStd{};

	let mut addr  = Addr::try_from( Sum(5), &mut exec ).expect( "spawn actor mailbox" );
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
	let mut exec = AsyncStd{};

	let sum = Sum(5);

	let (tx, rx) = mpsc::unbounded();

	// Create mailbox
	//
	let name = Some( "Sum".into() )                      ;
	let mb   = Inbox::new( name.clone(), Box::new(rx) )  ;
	let id   = mb.id()                                   ;
	let mut addr  = Addr ::new( id, name, Box::new(tx) ) ;
	let mut addr2 = addr.clone()                         ;

	mb.start( sum, &mut exec ).expect( "Failed to start mailbox" );

	addr .call( Add( 10 ) ).await.expect( "Send failed" );
	addr2.call( Add( 10 ) ).await.expect( "Send failed" );

	let resp = addr.call ( Show{} ).await.expect( "Call failed" );

	assert_eq!( 25, resp );
}
