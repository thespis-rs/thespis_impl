// Tested:
//
// - ✔ let thread of mailbox crash and verify mailbox closed error
// - ✔ let thread of mailbox crash and verify mailbox closed before response error
//
mod common;

use
{
	async_executors :: { *                                } ,
	thespis         :: { *                                } ,
	thespis_impl    :: { *                                } ,
	common          :: { actors::{ Sum, Add }, import::*  } ,
	futures         :: { channel::oneshot, task::SpawnExt } ,
	std             :: { panic::AssertUnwindSafe          } ,
};



// What we need to achieve here is that the mailbox is spawned, then closed while we
// keep the addr to try and Send. We should then get a MailboxClosed error.
//
#[async_std::test]
//
async fn test_mb_closed()
{
	// Create mailbox
	//
	let (tx, rx) = mpsc::unbounded()                        ;
	let name     = Some( "Sum".into() )                     ;
	let mb       = Inbox::new( name.clone(), Box::new(rx) ) ;
	let id       = mb.id()                                  ;
	let tx       = Box::new(tx.sink_map_err( |e| Box::new(e) as SinkError ));
	let mut addr = Addr ::new( id, name, tx )     ;

	let (trigger_tx, trigger_rx) = oneshot::channel();

	let mb_task = async move
	{
		mb.start_fut( Sum(5) ).await;

		trigger_tx.send(()).expect( "send trigger" );
	};

	let mb_handle = AsyncStd.spawn_handle( mb_task ).expect( "spawn mailbox" );

	addr.call( Add(10) ).await.expect( "call" );

	drop( mb_handle );

	// since we drop the handle, it's gonna fire not because of the send, but
	// because it get's dropped, so we don't care for success here.
	//
	let _ = trigger_rx.await;

	match addr.send( Add(10) ).await
	{
		Ok (_) => assert!( false, "Addr::send should fail" ),

		Err(e) => if let ThesErr::MailboxClosed{..} = e
		{
			assert!( true )
		}

		else
		{
			assert!( false, "Wrong error returned: {:?}", e )
		},
	}
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
async fn test_mb_closed_before_response()
{
	// flexi_logger::Logger::with_str( "warn, thespis_impl=trace" ).start().expect( "flexi_logger");

	// Create mailbox
	//
	let (tx, rx) = mpsc::unbounded()                        ;
	let name     = Some( "Sum".into() )                     ;
	let mb       = Inbox::new( name.clone(), Box::new(rx) ) ;
	let id       = mb.id()                                  ;
	let tx       = Box::new(tx.sink_map_err( |e| Box::new(e) as SinkError ));
	let mut addr = Addr ::new( id, name, tx )     ;


	let mb_task = async move
	{
		let _ = AssertUnwindSafe( mb.start_fut( Panic ) ).catch_unwind().await;
	};

	AsyncStd.spawn( mb_task ).expect( "spawn mailbox" );


	match addr.call( Void ).await
	{
		Ok (_) => assert!( false, "Call should fail" ),

		Err(e) =>
		{
			if let ThesErr::ActorStoppedBeforeResponse{..} = e {}

			else
			{
				assert!( false, "Wrong error returned: {:?}", e )
			}
		}
	}
}



