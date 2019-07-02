#![ feature( async_await, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, optin_builtin_traits ) ]

// TODO:
//
// - ✔ let thread of mailbox crash and verify mailbox closed error
// - ✔ let thread of mailbox crash and verify mailbox closed before response error
// - mailbox full (need bounded channels for that)
//
mod common;

use
{
	thespis       :: { *                    } ,
	log           :: { *                    } ,
	thespis_impl  :: { *                    } ,
	async_runtime :: { rt                   } ,
	std           :: { thread               } ,
	common        :: { actors::{ Sum, Add } } ,
};



async fn mb_closed()
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new(             );
	let mut addr             = Addr ::new( mb.sender() );


	// TODO: find a way to terminate thread without panic
	//
	let _ = thread::spawn( move ||
	{
		mb.start( sum ).expect( "Failed to start mailbox" );
		panic!( "terminate thread" );

	}).join();


	match addr.send( Add(10) ).await
	{
		Ok (_) => assert!( false, "Addr::send should fail" ),

		Err(e) => if let ThesErrKind::MailboxClosed{..} = e.kind()
		{
			assert!( true )
		}

		else
		{
			assert!( false, "Wrong error returned: {:?}", e )
		},
	}
}


// This works because the thread program only spawns when the outer task is already running, so
// the mailbox will only be closed after the outer task runs. Hence, the send actually works,
// but the response is never built.
//
async fn mb_closed_before_response()
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new(             );
	let mut addr             = Addr ::new( mb.sender() );

	thread::spawn( move ||
	{
		mb.start( sum ).expect( "Failed to start mailbox" );
		panic!( "terminate thread" );
	});


	match  addr.call( Add(10) ).await
	{
		Ok (_) => assert!( false, "Call should fail" ),

		Err(e) => if let ThesErrKind::MailboxClosedBeforeResponse{..} = e.kind()
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

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}



#[test]
//
fn test_mb_closed_before_response()
{
	let program = async move
	{
		// let _ = simple_logger::init();

		trace!( "start program" );

		mb_closed_before_response().await;
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}



