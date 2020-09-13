// This benchmark allows profiling where the performance is used. It contains an outer actor which has
// to do an async operation in it's handler and an inner actor which just does a sync addition of u64.
//
use
{
	std:: { thread, sync::{ atomic::{ Ordering::*, AtomicU64 }, Arc } } ,
};


const MESSAGES: usize = 10_000_000;



struct Sum
{
	pub total: AtomicU64,
	pub inner: Arc<SumIn>,
}


struct SumIn
{
	pub count: AtomicU64,
}


struct Add ( u64 );



impl Sum
{
	async fn add( &self, msg: Add )
	{
		let inner = self.inner.show().await;

		self.total.fetch_add( msg.0 + inner, SeqCst );
	}

	async fn show( &self ) -> u64
	{
		self.total.load( SeqCst )
	}
}


impl SumIn
{
	async fn show( &self ) -> u64
	{
		let count = self.count.fetch_add( 1, SeqCst );
		count + 1
	}
}


fn main()
{
	let sum_in = Arc::new( SumIn{ count: AtomicU64::new( 0 ) } );
	let sum    = Arc::new( Sum{ total: AtomicU64::new( 5 ), inner: sum_in } );
	let sum2   = sum.clone();
	let (tx, rx) = futures::channel::oneshot::channel();

	let sum_thread = thread::spawn( move ||
	{
		async_std::task::block_on( async move
		{
			for _ in 0..MESSAGES
			{
				sum.add( Add( 10 ) ).await;
			}

			tx.send(()).expect( "send oneshot" );
		})
	});


	async_std::task::block_on( async move
	{
		rx.await.expect( "oneshot" );

		let res = sum2.show().await;

		dbg!( res );

		assert_eq!( MESSAGES as u64 *10 + 5 + termial( MESSAGES as u64 ), res );
	});


	sum_thread  .join().expect( "join sum    thread" );
}


fn termial( n: u64 ) -> u64
{
	n * ( n + 1 ) / 2
}
