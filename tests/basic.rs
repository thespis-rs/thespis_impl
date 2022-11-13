// Tested:
//
// ✔ Use send and verify the actor has processed the message.
// ✔ Use call and verify the actor has processed the message.
// ✔ Use send from several addresses and verify the actor has processed the message.
// ✔ Use call from several addresses and verify the actor has processed the message.
//
#![ cfg(not( target_arch = "wasm32" )) ]


mod common;

use
{
	common          :: { import::*, actors::* } ,
	async_executors :: { AsyncStd             } ,
};



#[async_std::test]
//
async fn basic_send() -> Result<(), DynError >
{
	let mut addr = Addr::builder( "basic_send" ).spawn( Sum(5), &AsyncStd )?;

	addr.send( Add( 10 ) ).await?;

	assert_eq!( 15, addr.call( Show ).await? );

	Ok(())
}



#[async_std::test]
//
async fn basic_call() -> Result<(), DynError >
{
	let mut addr = Addr::builder( "basic_call" ).spawn( Sum(5), &AsyncStd )?;

	addr.call( Add(10) ).await?;

	assert_eq!( 15, addr.call( Show ).await? );

	Ok(())
}



#[async_std::test]
//
async fn send_from_multiple_addrs() -> Result<(), DynError >
{
	let mut addr  = Addr::builder( "send_from_multiple_addrs").spawn( Sum(5), &AsyncStd )?;
	let mut addr2 = addr.clone();

	addr .send( Add( 10 ) ).await?;
	addr2.send( Add( 10 ) ).await?;

	assert_eq!( 25, addr.call( Show{} ).await? );

	Ok(())
}



#[async_std::test]
//
async fn call_from_multiple_addrs() -> Result<(), DynError >
{
	let mut addr  = Addr::builder( "call_from_multiple_addrs" ).spawn( Sum(5), &AsyncStd )?;
	let mut addr2 = addr.clone();

	addr .call( Add( 10 ) ).await?;
	addr2.call( Add( 10 ) ).await?;

	assert_eq!( 25, addr.call ( Show{} ).await? );

	Ok(())
}
