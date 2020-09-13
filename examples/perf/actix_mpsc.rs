mod common;
use common::*;


#[ actix_rt::main ]
//
async fn main() -> Result< (), DynError >
{
	let (tx, rx) = oneshot::channel();


	// Create SumIn.
	//
	let builder = thread::Builder::new().name( "sum_in".to_string() );

	let _sum_in_thread = builder.spawn( move ||
	{
		System::run( ||
		{
			let sum_in = SumIn{ count: 0 };

			tx.send( sum_in.start() ).unwrap();
		})
	})?;

	let sum_in_addr = rx.await?;
	let (tx, rx) = oneshot::channel();


	// Create Sum.
	//
	let builder = thread::Builder::new().name( "sum".to_string() );

	let _sum_thread = builder.spawn( move ||
	{
		System::run( ||
		{
			let sum = ActixSum{ total: 5, inner: sum_in_addr, _nosend: PhantomData };

			tx.send( sum.start() ).unwrap();
		})
	})?;

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
			System::run( ||
			{
				Arbiter::spawn( async move
				{
					for _ in 0..MESSAGES/MPSC_SENDERS
					{
						sum_addr2.send( Add(10) ).await.expect( "Send failed" );
					}

					System::current().stop();
				});
			})

		})?);
	}

	for sender in senders.into_iter()
	{
		sender.join().expect( "join sender thread" )?;
	}

	let res = sum_addr.send( Show{} ).await.expect( "Call failed" );

	assert_eq!( MPSC_TOTAL, res as usize );

	// sum_in_thread.join().expect( "join arbiter thread" )?;
	// sum_thread   .join().expect( "join arbiter thread" )?;

	actix_rt::System::current().stop();

	Ok(())
}
