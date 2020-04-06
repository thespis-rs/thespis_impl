// This benchmark allows profiling where the performance is used. It contains an outer actor which has
// to do an async operation in it's handler and an inner actor which just does a sync addition of u64.
//
use
{
	actix::{ prelude::*, fut::wrap_future },
};


// const BOUNDED : usize = 16;
const MESSAGES: usize = 100_000;



struct Sum
{
	pub total: u64,
	pub inner: Addr<SumIn>,
}


struct SumIn
{
	pub count: u64,
}

impl Actor for Sum   { type Context = Context<Self>; }
impl Actor for SumIn { type Context = Context<Self>; }

struct Add ( u64 );
struct Show       ;

impl Message for Add  { type Result = () ; }
impl Message for Show { type Result = u64; }


impl Handler< Add > for Sum
{
	type Result = actix::ResponseActFuture< Self, <Add as Message>::Result >;

	fn handle( &mut self, msg: Add, _ctx: &mut Self::Context ) -> Self::Result
	{
		let action = self.inner.send( Show );

		let act_fut = wrap_future::<_, Self>(action);

		let update_self = act_fut.map( move |result, actor, _ctx|
		{
			actor.total += msg.0 + result.expect( "Call SumIn" );
		});

		Box::pin( update_self )
	}
}


impl Handler< Show > for Sum
{
	type Result = u64;

	fn handle( &mut self, _msg: Show, _ctx: &mut actix::Context<Self> ) -> Self::Result
	{
		self.total
	}
}


impl Handler< Show > for SumIn
{
	type Result = u64;

	fn handle( &mut self, _msg: Show, _ctx: &mut actix::Context<Self> ) -> Self::Result
	{
		self.count += 1;
		self.count
	}
}


#[ actix_rt::main ]
//
async fn main()
{
	let sum_in_thread = Arbiter::new();
	let sum_thread    = Arbiter::new();

	let sum_in = SumIn{ count: 0 };
	let sum_in_addr = SumIn::start_in_arbiter( &sum_in_thread, |_| sum_in );

	let sum = Sum{ total: 5, inner: sum_in_addr };
	let sum_addr = Sum::start_in_arbiter( &sum_thread, |_| sum );


	for _ in 0..MESSAGES
	{
		sum_addr.send( Add( 10 ) ).await.expect( "Send failed" );
	}

	let res = sum_addr.send( Show{} ).await.expect( "Call failed" );

	dbg!( res );

	assert_eq!( MESSAGES as u64 *10 + 5 + termial( MESSAGES as u64 ), res );

	System::current().stop();
}


fn termial( n: u64 ) -> u64
{
	n * ( n + 1 ) / 2
}
