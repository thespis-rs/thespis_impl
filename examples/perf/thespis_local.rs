// This benchmark allows profiling where the performance is used. It contains an outer actor which has
// to do an async operation in it's handler and an inner actor which just does a sync addition of u64.
//
mod common;
use common::*;


fn main() -> Result< (), DynError >
{
	let exec = TokioCtBuilder::new().build().unwrap();

	let sum_in = SumIn{ count: 0 };
	let sum_in_addr = Addr::builder().bounded( Some(BOUNDED) ).start_local( sum_in, &exec )?;

	let     sum      = Sum{ total: 5, inner: sum_in_addr, _nosend: PhantomData };
	let mut sum_addr = Addr::builder().bounded( Some(BOUNDED) ).start_local( sum   , &exec )?;


	exec.block_on( async move
	{
		for _ in 0..MESSAGES
		{
			// sum_addr.send( Add( 10 ) ).await.expect( "Send failed" );
			sum_addr.call( Add( 10 ) ).await.expect( "Send failed" );
		}

		let res = sum_addr.call( Show{} ).await.expect( "Call failed" );

		dbg!( res );

		assert_eq!( MESSAGES *10 + 5 + termial( MESSAGES ), res as usize );
	});

	Ok(())
}
