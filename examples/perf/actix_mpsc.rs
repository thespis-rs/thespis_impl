mod common;
use common::*;


#[ actix_rt::main ]
//
async fn main() -> Result< (), DynError >
{
	let (tx, rx) = oneshot::channel();


	// Create SumIn.
	//
	let sum_in_thread = Arbiter::new();

	sum_in_thread.spawn_fn( move ||
	{
		let sum_in = SumIn{ count: 0 };

		tx.send( sum_in.start() ).unwrap();
	});

	let sum_in_addr = rx.await?;
	let (tx, rx) = oneshot::channel();


	// Create Sum.
	//
	let sum_thread = Arbiter::new();

	sum_thread.spawn_fn( move ||
	{
		let sum = ActixSum{ total: 5, inner: sum_in_addr, _nosend: PhantomData };

		tx.send( sum.start() ).unwrap();
	});

	let sum_addr = rx.await?;


	// Create sender threads.
	//
	let mut senders = Vec::with_capacity( MPSC_SENDERS );

	for _ in 0..MPSC_SENDERS
	{
		let sum_addr2 = sum_addr.clone();

		let builder = thread::Builder::new().name( "sender".to_string() );

		senders.push( builder.spawn( move ||
		{
			block_on ( async
			{
				for _ in 0..MESSAGES/MPSC_SENDERS
				{
					sum_addr2.send( Add(10) ).await.expect( "Send failed" );
				}
			});

		})?);
	}


	for sender in senders.into_iter()
	{
		sender.join().expect( "join sender thread" );
	}

	let res = sum_addr.send( Show{} ).await.expect( "Call failed" );

	assert_eq!( MPSC_TOTAL, dbg!(res) as usize );

	sum_in_thread.stop();
	sum_thread   .stop();

	sum_in_thread.join().expect( "join arbiter thread" );
	sum_thread   .join().expect( "join arbiter thread" );

	Ok(())
}
