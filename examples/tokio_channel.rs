//! Illustration of how to use tokio channels.
//
use
{
	tokio_util      :: { sync::PollSender         } ,
	tokio_stream    :: { wrappers::ReceiverStream } ,
	thespis         :: { *                        } ,
	thespis_impl    :: { *                        } ,
	async_executors :: { AsyncStd                 } ,
	std             :: { error::Error             } ,
};


#[ derive( Actor ) ]
//
struct MyActor;
struct Ping;

impl Message for Ping
{
	type Return = &'static str;
}


impl Handler< Ping > for MyActor
{
	#[async_fn]	fn handle( &mut self, _msg: Ping ) -> &'static str
	{
		"pong"
	}
}


#[async_std::main]
//
async fn main() -> Result< (), Box<dyn Error> >
{
	let (tx,  rx) = tokio::sync::mpsc::channel( 16 );
	let rx = ReceiverStream::new(rx);

	let tx = PollSender::new(tx).sink_map_err( |_|
	{
		// The error from tokio-util is not Sync, because it contains the message.
		// This is a problem because we don't require messages to be `Sync`, however
		// we do want our error type to be `Send` + `Sync`, so it can't have the message.
		// If we would not require `Sync` on the error type, it couldn't be send across
		// threads unless it was `Clone`, but we don't require the user's messages to
		// be clone either...
		//
		// We don't count on recovering the message and construct a
		// simple io error here. Note that we could have wanted to use ThesErr::MailboxClosed,
		// but that requires ActorInfo and as we haven't spawned our actor yet, we don't really
		// know our ActorInfo. You can still do it by not using the builder and first creating
		// the Mailbox manually, however I don't think it's worth it.
		//
		std::io::Error::from(std::io::ErrorKind::NotConnected)
	});

	// We pass in our custom channel here as long as it implements `Sink + Clone + Unpin + 'static`
	// on the sender and `Stream + Unpin + 'static` on the receiver.
	//
	let (mut addr, mb_handle) = Addr::builder( "powerd by tokio" )
		.channel( tx, rx )
		.spawn_handle( MyActor, &AsyncStd )?
	;

	let result = addr.call( Ping ).await?;

	assert_eq!( "pong", result );
	dbg!( result );

	// will cause the mailbox future to end.
	//
	drop( addr );

	mb_handle.await;

	Ok(())
}
