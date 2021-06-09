// Tested:
//
// ✔ verify actor stops when all adresses are dropped.
// ✔ verify actor stops when all adresses are dropped before the mailbox was even started.
//
#![ cfg(not( target_arch = "wasm32" )) ]

mod common;

use
{
	common          :: { import::*, actors::* } ,
	async_executors :: { AsyncStd                } ,
};



#[async_std::test]
//
async fn stop_when_adresses_dropped_before_start_mb() -> Result<(), DynError >
{
	// let _ = flexi_logger::Logger::with_str( "trace" ).start();

	let (addr, mb) = Addr::builder().build();

	let addr2 = addr.clone();

	drop( addr  );
	drop( addr2 );

	let mb_handle = AsyncStd.spawn_handle( mb.start( Sum(5) ) )?;

	// the handle will resolve when the mailbox has ended. If not this test will hang.
	//
	mb_handle.await;

	Ok(())
}


#[async_std::test]
//
async fn stop_when_adresses_dropped() -> Result<(), DynError >
{
	// let _ = flexi_logger::Logger::with_str( "trace" ).start();

	let (mut addr, mb) = Addr::builder().build();

	let mb_handle = AsyncStd.spawn_handle( mb.start( Sum(5) ) )?;

	let addr2 = addr.clone();

	// call guarantees that the message has been processed
	//
	addr.call( Add( 10 ) ).await?;

	drop( addr  );
	drop( addr2 );

	// the handle will resolve when the mailbox has ended. If not this test will hang.
	//
	mb_handle.await;

	Ok(())
}

