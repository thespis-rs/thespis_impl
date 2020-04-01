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
	let exec = AsyncStd{};

	let (tx, rx) = mpsc::channel( 5 )                       ;
	let name     = Some( "Sum".into() )                     ;
	let mb       = Inbox::new( name.clone(), Box::new(rx) ) ;
	let id       = mb.id()                                  ;
	let mut addr = Addr ::new( id, name, Box::new(tx) )     ;

	mb.start( Sum(5), &exec ).expect( "spawn actor" );

	addr.send( Add( 10 ) ).await.expect( "Send failed" );

	let result = addr.call( Show{} ).await.expect( "Call failed" );

	assert_eq!( 15, result );
}



#[async_std::test]
//
async fn test_basic_call()
{
	let exec = AsyncStd{};

	let (tx, rx) = mpsc::channel( 5 )                       ;
	let name     = Some( "Sum".into() )                     ;
	let mb       = Inbox::new( name.clone(), Box::new(rx) ) ;
	let id       = mb.id()                                  ;
	let mut addr = Addr ::new( id, name, Box::new(tx) )     ;

	mb.start( Sum(5), &exec ).expect( "spawn actor" );

	addr.call( Add(10) ).await.expect( "Send failed" );

	let result = addr.call( Show{} ).await.expect( "Call failed" );

	assert_eq!( 15, result );
}



#[async_std::test]
//
async fn send_from_multiple_addrs()
{
	let exec = AsyncStd{};

	let (tx, rx)  = mpsc::channel( 5 )                       ;
	let name      = Some( "Sum".into() )                     ;
	let mb        = Inbox::new( name.clone(), Box::new(rx) ) ;
	let id        = mb.id()                                  ;
	let mut addr  = Addr ::new( id, name, Box::new(tx) )     ;
	let mut addr2 = addr.clone()                             ;

	mb.start( Sum(5), &exec ).expect( "spawn actor" );


	addr .send( Add( 10 ) ).await.expect( "Send failed" );
	addr2.send( Add( 10 ) ).await.expect( "Send failed" );

	let resp = addr.call( Show{} ).await.expect( "Call failed" );

	assert_eq!( 25, resp );
}



#[async_std::test]
//
async fn call_from_multiple_addrs()
{
	let exec = AsyncStd{};

	let (tx, rx)  = mpsc::channel( 5 )                       ;
	let name      = Some( "Sum".into() )                     ;
	let mb        = Inbox::new( name.clone(), Box::new(rx) ) ;
	let id        = mb.id()                                  ;
	let mut addr  = Addr ::new( id, name, Box::new(tx) )     ;
	let mut addr2 = addr.clone()                             ;

	mb.start( Sum(5), &exec ).expect( "spawn actor" );

	addr .call( Add( 10 ) ).await.expect( "Send failed" );
	addr2.call( Add( 10 ) ).await.expect( "Send failed" );

	let resp = addr.call ( Show{} ).await.expect( "Call failed" );

	assert_eq!( 25, resp );
}
