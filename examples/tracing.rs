//! Check how tracing logs log statements from our actors.
//!
//! We create 2 named actors of the same types, and log from a handler.
//! We can see that we always have the actor name and id to understand which actor is logging.
//!
//! Note that you can use tracing_prism to see concurrent logs side by side:
//! https://github.com/najamelan/tracing_prism
//!
use
{
	thespis           :: { *            } ,
	thespis_impl      :: { *            } ,
	async_executors   :: { AsyncStd     } ,
	std               :: { error::Error } ,
	tracing           :: { *            } ,
};


#[ derive( Actor ) ]
//
struct MyActor;


// Message types can have data of course, but in general should be UnwindSafe an Send.
// It's generally a code smell if you put anything that allows shared mutability in an
// actor message.
//
struct Ping;


// The Message trait really represents a protocol. So where it's request-response, the
// response type is fixed here, not in Handler.
//
impl Message for Ping
{
	type Return = &'static str;
}


impl Handler< Ping > for MyActor
{
	// This macro does something similar to `async-trait`, but much simpler,
	// and compatible with a hand written version. Look at the desugar example
	// to see the manual equivalent.
	//
	#[async_fn]	fn handle( &mut self, _msg: Ping ) -> &'static str
	{
		info!( "Hi from MyActor" );
		"pong"
	}
}


#[async_std::main]
//
async fn main() -> Result< (), Box<dyn Error> >
{
	tracing_subscriber::fmt::Subscriber::builder()

		// .with_timer( tracing_subscriber::fmt::time::ChronoLocal::rfc3339() )
	   // .with_max_level(tracing::Level::TRACE)
	   // .json() // use json format if you want a nicer view on tracing_prism
	   .with_env_filter( "trace,polling=warn,async_io=warn,async_std::warn" )
	   .init()
	;


	// This is a convenience API for creating and starting an actor. For a
	// structured concurrency approach, use `start_handle`, so you can await
	// the output of the mailbox.
	//
	let mut addr  = Addr::builder( "Alice" ).spawn( MyActor, &AsyncStd )?;
	let mut addr2 = Addr::builder( "Bob"   ).spawn( MyActor, &AsyncStd )?;

	// Call is request-response, as opposed to `send` which comes from `Sink`
	// and is a one way message.
	//
	let result  = addr .call( Ping ).await?;
	let result2 = addr2.call( Ping ).await?;

	assert_eq!( "pong", result  );
	assert_eq!( "pong", result2 );

	dbg!( result );
	dbg!( result2 );

	Ok(())
}
