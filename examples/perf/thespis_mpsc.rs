mod common;
use common::*;


#[tokio::main]
//
async fn main() -> Result< (), DynError >
{
	let (sum_in_addr , sum_in_mb) = Addr::builder( "sum_in" ).bounded( Some(MPSC_BOUNDED) ).build() ;
	let (mut sum_addr, sum_mb   ) = Addr::builder( "sum"    ).bounded( Some(MPSC_BOUNDED) ).build() ;


	// Create sender threads.
	//
	let mut senders = Vec::with_capacity( MPSC_SENDERS );

	for _ in 0..MPSC_SENDERS
	{
		let mut sum_addr2 = sum_addr.clone();

		let builder = thread::Builder::new().name( "sender".to_string() );

		senders.push( builder.spawn( move ||
		{
			let exec = TokioCtBuilder::new().build().unwrap();

			exec.block_on( async move
			{
				for _ in 0..MESSAGES/MPSC_SENDERS
				{
					sum_addr2.send( Add(10) ).await.unwrap();
				}
			});

		})?);
	}


	// Create SumIn
	//
	let builder = thread::Builder::new().name( "sum_in".to_string() );

	let sum_in_thread = builder.spawn( move ||
	{
		let exec = TokioCtBuilder::new().build().unwrap();

		let sum_in = SumIn { count: 0 };

		exec.block_on( sum_in_mb.start_local(sum_in) )
	})?;


	// Create Sum
	//
	let builder = thread::Builder::new().name( "sum".to_string() );

	let sum_thread = builder.spawn( move ||
	{
		let exec = TokioCtBuilder::new().build().unwrap();

		let sum = Sum { total: 5, inner: sum_in_addr, _nosend: PhantomData } ;

		exec.block_on( sum_mb.start_local(sum) );
	})?;


	// Join Sender threads.
	//
	for sender in senders.into_iter()
	{
		sender.join().unwrap();
	}


	// Verify result.
	//
	let res = sum_addr.call( Show ).await?;
	drop( sum_addr );

	assert_eq!( MPSC_TOTAL, res as usize );

	sum_in_thread.join().unwrap();
	sum_thread   .join().unwrap();

	Ok(())
}
