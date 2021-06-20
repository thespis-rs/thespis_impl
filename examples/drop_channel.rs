//! Use a drop channel between address and mailbox. Only the most recent message will
//! be kept. Older messages will be overwritten.
//!
//! This will send 100k messages and print how many were actually processed. The rest
//! was overwritten.
//
use
{
	thespis           :: { *                                       } ,
	thespis_impl      :: { *                                       } ,
	futures::executor :: { block_on                                } ,
	std               :: { error::Error, num::NonZeroUsize, thread } ,
	ring_channel      :: { *                                       } ,
};


const MPSC_BOUNDED: usize = 1;
const MPSC_SENDERS: usize = 24;
const MESSAGES    : usize = 100_000;


#[ derive( Actor ) ]
//
struct Accu
{
	count: usize,
}

struct Count;

impl Message for Count { type Return = (); }

struct Show;

impl Message for Show { type Return = usize; }


impl Handler< Count > for Accu
{
	#[async_fn]	fn handle( &mut self, _msg: Count )
	{
		self.count += 1;
	}
}


impl Handler< Show > for Accu
{
	#[async_fn]	fn handle( &mut self, _msg: Show ) -> usize
	{
		self.count
	}
}


#[async_std::main]
//
async fn main() -> Result< (), Box<dyn Error> >
{
	let (tx, rx) = ring_channel( NonZeroUsize::new(MPSC_BOUNDED).unwrap() );


	let tx = tx.sink_map_err( |_|
	{
		// The error from ring_channel is not Sync, because it contains the message.
		// This is a problem because we don't require messages to be Sync. That would
		// imply making thespis unsafe by addin a Sync impl to our wrapper for messages
		// on the channel.
		//
		// Alternatively, we just don't count on recovering the message and construct a
		// simple io error here. Note that we could have wanted to use ThesErr::MailboxClosed,
		// but that requires ActorInfo and as we haven't spawned our actor yet, we don't really
		// know our ActorInfo. You can still do it by not using the builder and first creating
		// the Mailbox manually, however I don't think it's worth it.
		//
		let error = std::io::Error::from(std::io::ErrorKind::NotConnected);
		Box::new(error) as DynError
	});

	let (mut accu_addr , accu_mb) = Addr::builder()

		.channel( Box::new(tx), Box::new(rx) )
		.build() ;


	// Create sender threads.
	//
	let mut senders = Vec::with_capacity( MPSC_SENDERS );

	for _ in 0..MPSC_SENDERS
	{
		let mut accu_addr2 = accu_addr.clone();

		let builder = thread::Builder::new().name( "sender".to_string() );

		senders.push( builder.spawn( move ||
		{
			block_on( async move
			{
				for _ in 0..MESSAGES/MPSC_SENDERS
				{
					accu_addr2.send( Count ).await.unwrap();
				}
			});

		})?);
	}


	// Create SumIn
	//
	let builder = thread::Builder::new().name( "sum_in".to_string() );

	let accu_thread = builder.spawn( move ||
	{
		let accu = Accu { count: 0 };

		block_on( accu_mb.start_local(accu) )
	})?;


	// Join Sender threads.
	//
	for sender in senders.into_iter()
	{
		sender.join().unwrap();
	}


	// Verify result.
	//
	let res = accu_addr.call( Show ).await?;
	drop( accu_addr );

	println!( "Processed {} messages out of {}", res, MESSAGES );

	accu_thread.join().unwrap();

	Ok(())
}
