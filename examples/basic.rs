//! Illustration of the most simple actor exchange. Ping-Pong.
//
use
{
	thespis           :: { *            } ,
	thespis_impl      :: { *            } ,
	async_executors   :: { AsyncStd     } ,
	std               :: { error::Error } ,
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
		"pong"
	}
}


#[async_std::main]
//
async fn main() -> Result< (), Box<dyn Error> >
{
	// This is a convenience API for creating and starting an actor. For a
	// structured concurrency approach, use `start_handle`, so you can await
	// the output of the mailbox.
	//
	let mut addr = Addr::builder().spawn( MyActor, &AsyncStd )?;

	// Call is request-response, as opposed to `send` which comes from `Sink`
	// and is a one way message.
	//
	let result = addr.call( Ping ).await?;

	assert_eq!( "pong", result );
	dbg!( result );

	Ok(())
}
