// This benchmark allows profiling where the performance is used. It contains an outer actor which has
// to do an async operation in it's handler and an inner actor which just does a sync addition of u64.
//
mod common;
use common::*;


// const BOUNDED : usize = 16;
const MESSAGES: usize = 100_000;


#[ actix_rt::main ]
//
async fn main()
{
	let sum_in_thread = Arbiter::new();
	let sum_thread    = Arbiter::new();

	let sum_in      = SumIn { count: 0 };
	let sum_in_addr = SumIn::start_in_arbiter( &sum_in_thread, |_| sum_in );
	let sum         = ActixSum   { total: 5, inner: sum_in_addr };
	let sum_addr    = ActixSum  ::start_in_arbiter( &sum_thread   , |_| sum    );


	for _ in 0..MESSAGES
	{
		sum_addr.send( Add( 10 ) ).await.expect( "Send failed" );
	}

	let res = sum_addr.send( Show{} ).await.expect( "Call failed" );

	assert_eq!( MESSAGES *10 + 5 + termial( MESSAGES ), res as usize );

	System::current().stop();
}
