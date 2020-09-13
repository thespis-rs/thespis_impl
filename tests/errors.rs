// Tested:
//
// ✔ let thread of mailbox crash and verify mailbox closed error
// ✔ let thread of mailbox crash and verify mailbox closed before response error
//
mod common;

use
{
	common :: { *, actors::*, import::* } ,
	std    :: { panic::AssertUnwindSafe } ,
};



// What we need to achieve here is that the mailbox is spawned, then closed while we
// keep the addr to try and Send. We should then get a MailboxClosed error.
//
#[async_std::test]
//
async fn test_mb_closed() -> Result<(), DynError >
{
	let (mut addr, mb) = Addr::builder().build();

	let (trigger_tx, trigger_rx) = oneshot::channel();


	let mb_task = async move
	{
		mb.start( Sum(5) ).await;

		trigger_tx.send(()).expect( "send trigger" );
	};

	let mb_handle = AsyncStd.spawn_handle( mb_task )?;


	// Make sure the mailbox is actually up and running before moving on.
	//
	addr.call( Add(10) ).await?;

	drop( mb_handle );

	// since we drop the handle, it's gonna fire not because of the send, but
	// because it get's dropped, so we don't care for success here.
	//
	assert!( trigger_rx.await.is_err() );


	match addr.send( Add(10) ).await
	{
		Ok (_) => panic!( "Addr::send should fail" ),

		Err(e) => match e
		{
			ThesErr::MailboxClosed{..} => {}
			_                          => assert!( false, "Wrong error returned: {:?}", e ),
		}
	}

	Ok(())
}




#[ derive( Actor ) ] struct Panic;

struct Void;

impl Message for Void { type Return = (); }

impl Handler<Void> for Panic
{
	fn handle( &mut self, _: Void ) -> Return<'_, ()>
	{
		panic!();
	}
}


#[async_std::test]
//
async fn test_mb_closed_before_response() -> Result<(), DynError >
{
	// flexi_logger::Logger::with_str( "warn, thespis_impl=trace" ).start().expect( "flexi_logger");

	let (mut addr, mb) = Addr::builder().build();


	let mb_task = async move
	{
		let _ = AssertUnwindSafe( mb.start( Panic ) ).catch_unwind().await;
	};

	AsyncStd.spawn( mb_task )?;


	match addr.call( Void ).await
	{
		Ok (_) => panic!( "Call should fail" ),

		Err(e) => match e
		{
			ThesErr::ActorStoppedBeforeResponse{..} => {}
			_ => assert!( false, "Wrong error returned: {:?}", e ),
		}
	}

	Ok(())
}



