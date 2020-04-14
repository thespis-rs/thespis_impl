mod common;
use common::*;


#[ actix_rt::main ]
//
async fn main()
{
	let mut sum_in_thread = actix::Arbiter::new();
	let mut sum_thread    = actix::Arbiter::new();

	let sum_in      = SumIn{ count: 0 };
	let sum_in_addr = SumIn::start_in_arbiter( &sum_in_thread, |_| sum_in );

	let sum      = ActixSum{ total: 5, inner: sum_in_addr };
	let sum_addr = ActixSum::start_in_arbiter( &sum_thread, |_| sum );

	let mut senders = Vec::with_capacity( MPSC_SENDERS );

	for _ in 0..MPSC_SENDERS
	{
		let addr = sum_addr.clone();
		let arb  = actix::Arbiter::new();
		let arb2 = arb.clone();

		let fut = async move
		{
			for _ in 0..MESSAGES/MPSC_SENDERS
			{
				addr.send( Add(10) ).await.expect( "Send failed" );
			}

			arb2.stop();
		};

		arb.send( fut.boxed() );

		senders.push( arb );
	}

	for mut sender in senders.into_iter()
	{
		sender.join().expect( "join sender thread" );
	}

	let res = sum_addr.send( Show{} ).await.expect( "Call failed" );

	assert_eq!( MPSC_TOTAL, res as usize );

	sum_in_thread.stop();
	sum_thread   .stop();

	sum_in_thread.join().expect( "join arbiter thread" );
	sum_thread   .join().expect( "join arbiter thread" );

	actix_rt::System::current().stop();
}
