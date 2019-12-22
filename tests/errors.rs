#![ feature( optin_builtin_traits ) ]

// TODO:
//
// - ✔ let thread of mailbox crash and verify mailbox closed error
// - ✔ let thread of mailbox crash and verify mailbox closed before response error
// - mailbox full (need bounded channels for that)
//
mod common;

use
{
	thespis         :: { *                                                     } ,
	log             :: { *                                                     } ,
	thespis_impl    :: { *                                                     } ,
	std             :: { thread                                                } ,
	async_executors :: { *                                                     } ,
	common          :: { actors::{ Sum, Add }                                  } ,
	futures         :: { executor::block_on, future::FutureExt, task::SpawnExt } ,

};



// What we need to achieve here is that the mailbox is spawned, then closed while we
// keep the addr to try and Send. We should then get a MailboxClosed error.
//
// It's not quite clear what is the best way to do this, but this test passes for now
// on x86_64 linux...
//
// TODO: this doesn't quite work either. Sometimes the the tests fails.
//
async fn mb_closed()
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new( "Sum".into() );
	let mut addr             = Addr ::new( mb.sender()  );

	let (mb_fut, handle) = mb.start_fut( sum ).remote_handle();

	let exec = ThreadPool::new().expect( "create threadpool" );

	exec.spawn( mb_fut ).expect( "spawn mailbox" );

	// If we drop the handle in the same thread, the mb never actually runs.
	//
	thread::spawn( move ||
	{
		drop( handle );

	}).join().expect( "join thread" );


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




#[test]
//
fn test_mb_closed()
{
	let program = async move
	{
		// let _ = simple_logger::init();

		trace!( "start program" );

		mb_closed().await;
	};

	block_on( program );
}



// TODO: I don't manage to trigger MailboxClosedBeforeResponse...
//
// #[test]
// //
// fn test_mb_closed_before_response()
// {
// 	let program = async move
// 	{
// 		// let _ = simple_logger::init();

// 		trace!( "start program" );

// 		mb_closed_before_response().await;
// 	};

// 	block_on( program );
// }





// This works because the thread program only spawns when the outer task is already running, so
// the mailbox will only be closed after the outer task runs. Hence, the send actually works,
// but the response is never built.
//
// async fn mb_closed_before_response()
// {
// 	flexi_logger::Logger::with_str( "warn, thespis_impl=trace" ).start().expect( "flexi_logger");

// 	let sum = Sum(5);

// 	// Create mailbox
// 	//
// 	let     mb  : Inbox<Sum> = Inbox::new(             );
// 	let mut addr             = Addr ::new( mb.sender() );

// 	let (mb_fut, handle) = mb.start_fut( sum ).remote_handle();

// 	let mut exec = ThreadPool::new().expect( "create threadpool" );

// 	exec.spawn( mb_fut ).expect( "spawn mailbox" );


// 	// If we drop the handle in the same thread, the mb never actually runs.
// 	//
// 	thread::spawn( move ||
// 	{
// 		thread::yield_now();
// 		drop( handle );

// 	});


// 	match addr.call( Add(10) ).await
// 	{
// 		Ok (_) => assert!( false, "Call should fail" ),

// 		Err(e) =>
// 		{
// 			if let ThesErr::MailboxClosedBeforeResponse{..} = e {}

// 			else
// 			{
// 				assert!( false, "Wrong error returned: {:?}", e )
// 			}
// 		}
// 	}
// }
