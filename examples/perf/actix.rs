// This benchmark allows profiling where the performance is used. It contains an outer actor which has
// to do an async operation in it's handler and an inner actor which just does a sync addition of u64.
//
mod common;
use common::*;


// const BOUNDED : usize = 16;
const MESSAGES: usize = 100_000;


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


	for _ in 0..MESSAGES
	{
		sum_addr.send( Add( 10 ) ).await.expect( "Send failed" );
	}

	let res = sum_addr.send( Show{} ).await.expect( "Call failed" );

	assert_eq!( MESSAGES *10 + 5 + termial( MESSAGES ), res as usize );

	System::current().stop();

	Ok(())
}
