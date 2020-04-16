use
{
	thespis           :: { *            } ,
	thespis_impl      :: { *            } ,
	futures::executor :: { block_on     } ,
	std               :: { error::Error, num::NonZeroUsize, thread } ,
	ring_channel      :: { *            } ,
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
	let tx = tx.sink_map_err( |_| Box::new( thespis_impl::ThesErr::MailboxClosed{ actor: "".to_string() } ) as SinkError );

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

		block_on( accu_mb.start_fut_local(accu) )
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

	println!( "Processed {} messages out of {}", res, MESSAGES * MPSC_SENDERS );

	accu_thread.join().unwrap();

	Ok(())
}
