// This benchmark allows profiling where the performance is used. It contains an outer actor which has
// to do an async operation in it's handler and an inner actor which just does a sync addition of u64.
//
mod common;
use common::*;


// queue size for the bounded channel, 16 is same as actix default size.
//
const BOUNDED: usize = 16 ;
const SENDERS: usize = 1  ;


#[async_std::main]
//
async fn main()
{
	let (sum_in_addr, sum_in_mb) = Addr::builder().bounded( Some(BOUNDED) ).build() ;
	let (mut sum_addr, sum_mb)   = Addr::builder().bounded( Some(BOUNDED) ).build() ;

	let sum    = Sum  { total: 5, inner: sum_in_addr } ;
	let sum_in = SumIn{ count: 0 }                     ;

	let sumin_thread = thread::spawn( move ||
	{
		block_on( sum_in_mb.start_fut( sum_in ) );
	});

	let sum_thread = thread::spawn( move ||
	{
		block_on( sum_mb.start_fut( sum ) );
	});


	for _ in 0..MESSAGES/SENDERS
	{
		sum_addr.send( Add(10) ).await.expect( "Send failed" );
	}

	let res = sum_addr.call( Show{} ).await.expect( "Call failed" );
	drop(sum_addr);

	assert_eq!( MESSAGES *10 + 5 + termial( MESSAGES ), res as usize );


	sumin_thread.join().expect( "join sum_in thread" );
	sum_thread  .join().expect( "join sum    thread" );
}
