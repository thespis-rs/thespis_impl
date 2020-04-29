// This benchmark allows profiling where the performance is used. It contains an outer actor which has
// to do an async operation in it's handler and an inner actor which just does a sync addition of u64.
//
// It turns out that bounded channels are actually faster. Both tokio::sync and futures::mpsc::unbounded
// take 2x time compared to tokio::sync bounded with buffer size 16.
//
use
{
	async_chanx       :: { *                      } ,
	thespis           :: { *                      } ,
	thespis_impl      :: { *                      } ,
	std               :: { thread                 } ,
	tokio             :: { sync::mpsc             } ,
};


// queue size for the bounded channel, 16 is same as actix default size.
//
const MESSAGES: usize = 100_000;



#[ derive( Actor ) ] struct Sum
{
	pub total: u64,
	pub inner: Addr<SumIn>,
}


#[ derive( Actor ) ] struct SumIn
{
	pub count: u64,
}


struct Add ( u64 );
struct Show       ;

impl Message for Add  { type Return = () ; }
impl Message for Show { type Return = u64; }


impl Handler< Add > for Sum
{
	fn handle( &mut self, msg: Add ) -> Return<()> { Box::pin( async move
	{
		let inner = self.inner.call( Show ).await.expect( "call inner" );

		self.total += msg.0 + inner;

	})}
}


impl Handler< Show > for Sum
{
	fn handle( &mut self, _msg: Show ) -> Return<u64> { Box::pin( async move
	{

		self.total

	})}
}


impl Handler< Show > for SumIn
{
	fn handle( &mut self, _msg: Show ) -> Return<u64> { Box::pin( async move
	{

		self.count += 1;
		self.count

	})}
}


fn main()
{
	let (tx, rx)    = mpsc::unbounded_channel()                                                                ;
	let tx          = Box::new( TokioUnboundedSender::new( tx ).sink_map_err( |e| Box::new(e) as SinkError ) ) ;
	let sum_in_mb   = Inbox::new( None, Box::new( rx ) )                                                       ;
	let sum_in_addr = Addr::new( sum_in_mb.id(), sum_in_mb.name(), tx )                                        ;

	let (tx, rx)     = mpsc::unbounded_channel()                                                                ;
	let     tx       = Box::new( TokioUnboundedSender::new( tx ).sink_map_err( |e| Box::new(e) as SinkError ) ) ;
	let     sum_mb   = Inbox::new( None, Box::new( rx ) )                                                       ;
	let mut sum_addr = Addr::new( sum_mb.id(), sum_mb.name(), tx )                                              ;
	let     sum      = Sum{ total: 5, inner: sum_in_addr }                                                      ;

	let sumin_thread = thread::spawn( move ||
	{
		let sum_in = SumIn{ count: 0 };

		async_std::task::block_on( sum_in_mb.start( sum_in ) );
	});

	let sum_thread = thread::spawn( move ||
	{
		async_std::task::block_on( sum_mb.start( sum ) );
	});


	async_std::task::block_on( async move
	{
		for _ in 0..MESSAGES
		{
			// sum_addr.send( Add( 10 ) ).await.expect( "Send failed" );
			sum_addr.call( Add( 10 ) ).await.expect( "Send failed" );
		}

		let res = sum_addr.call( Show{} ).await.expect( "Call failed" );

		dbg!( res );

		assert_eq!( MESSAGES as u64 *10 + 5 + termial( MESSAGES as u64 ), res );
	});


	sumin_thread.join().expect( "join sum_in thread" );
	sum_thread  .join().expect( "join sum    thread" );
}


fn termial( n: u64 ) -> u64
{
	n * ( n + 1 ) / 2
}
