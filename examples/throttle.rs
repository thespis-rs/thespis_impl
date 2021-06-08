//! Demonstrates how we can throttle an actor. We use the stream_throttle crate to deliver
//! messages at a rate of 1 per second.
//
use
{
	thespis         :: { *                                           } ,
	thespis_impl    :: { SinkError, Mailbox                          } ,
	async_executors :: { AsyncStd                                    } ,
	std             :: { error::Error, time::Duration                } ,
	futures         :: { channel::mpsc, SinkExt                      } ,
	stream_throttle :: { ThrottleRate, ThrottlePool, ThrottledStream } ,
};


#[ derive( Debug, Actor ) ]
//
struct MyActor
{
	count: usize,
}


struct Count;
struct Show;

impl Message for Count { type Return = ();    }
impl Message for Show  { type Return = usize; }


impl Handler< Count > for MyActor
{
	#[async_fn] fn handle( &mut self, _msg: Count )
	{
		self.count += 1;
		println!( "Received a message." );
	}
}

impl Handler< Show > for MyActor
{
	#[async_fn] fn handle( &mut self, _msg: Show ) -> usize
	{
		self.count
	}
}


#[async_std::main]
//
async fn main() -> Result< (), Box<dyn Error> >
{
	let (tx, rx)  = mpsc::channel( 10 );

	let rate = ThrottleRate::new( 1, Duration::from_secs(1) );
	let pool = ThrottlePool::new( rate );
	let rx   = rx.throttle( pool );

	let tx = Box::new( tx.sink_map_err( |e| Box::new(e) as SinkError ) );
	let mb = Mailbox::new( Some("Throttled"), Box::new(rx) );
	let mut addr = mb.addr( tx );


	let mb_handle = mb.start_handle( MyActor{ count: 0 }, &AsyncStd )?;


	for _ in 0..10
	{
		addr.send( Count ).await?;
	}


	assert_eq!( 10, addr.call(Show).await? );

	// Allow the program to end.
	//
	// One gotcha here. Normally the mailbox will stay alive even after dropping all strong
	// addresses as long as there are messages in the channel. However, when the channel returns
	// `Pending` it thinks it's empty. Where as here it is just throttled.
	//
	// This is not an issue here because we use the call with `Show` above to synchronize. That message
	// needs to be handled before we can arrive here, which means all prior messages have been handled as
	// well.
	//
	// If it wasn't for this last call, the program would end after the actor processed just 1 message.
	//
	drop( addr );

	mb_handle.await;

	Ok(())
}
