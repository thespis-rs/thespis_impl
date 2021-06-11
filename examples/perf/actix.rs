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


	for _ in 0..MESSAGES
	{
		sum_addr.send( Add( 10 ) ).await.expect( "Send failed" );
	}

	let res = sum_addr.send( Show{} ).await.expect( "Call failed" );

	assert_eq!( MESSAGES *10 + 5 + termial( MESSAGES ), dbg!(res) as usize );


	sum_thread.stop();
	sum_thread.join().unwrap();

	sum_in_thread.stop();
	sum_in_thread.join().unwrap();

	Ok(())
}
