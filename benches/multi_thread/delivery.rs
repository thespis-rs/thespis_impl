use
{
	async_executors   :: { * } ,
	criterion         :: { Criterion, Benchmark, criterion_group, criterion_main           } ,
	futures           :: { executor::{ block_on }, executor::{ LocalPool }, task::SpawnExt } ,
	thespis           :: { *                                                               } ,
	thespis_impl      :: { *                                                               } ,
	std               :: { thread, sync::{ Arc, atomic::{ AtomicU64, Ordering } }          } ,
};



#[ derive( Actor ) ] struct Sum( u64 );

struct Add (u64);
struct Show     ;

impl Message for Add  { type Return = () ; }
impl Message for Show { type Return = u64; }


impl Handler< Add > for Sum
{
	fn handle( &mut self, msg: Add ) -> Return<()> { Box::pin( async move
	{

		self.0 += msg.0;

	})}
}


impl Handler< Show > for Sum
{
	fn handle( &mut self, _msg: Show ) -> Return<u64> { Box::pin( async move
	{

		self.0

	})}
}



fn send_threadpool()
{
	let bench = async move
	{
		let     sum  = Sum(5)                                          ;
		let     mb   = Inbox::new()                                    ;
		let mut addr = Addr::new( mb.sender() )                        ;
		let mut pool = ThreadPool::new().expect( "create threadpool" ) ;

		// This is ugly right now. It will be more ergonomic in the future.
		//
		pool.spawn( mb.start_fut( sum ) ).expect( "Spawning failed" );

		for _i in 0..100usize
		{
			addr.send( Add( 10 ) ).await.expect( "Send failed" );
		}

		let res = addr.call( Show{} ).await.expect( "Call failed" );
		assert_eq!( 1005, res );
	};

	block_on( bench );
}



fn call_threadpool()
{
	let bench = async move
	{
		let     sum  = Sum(5)                                          ;
		let     mb   = Inbox::new()                                    ;
		let mut addr = Addr::new( mb.sender() )                        ;
		let mut pool = ThreadPool::new().expect( "create threadpool" ) ;

		// This is ugly right now. It will be more ergonomic in the future.
		//
		pool.spawn( mb.start_fut( sum ) ).expect( "Spawning failed" );

		for _i in 0..100usize
		{
			addr.call( Add( 10 ) ).await.expect( "Send failed" );
		}

		let res = addr.call( Show{} ).await.expect( "Call failed" );
		assert_eq!( 1005, res );
	};

	block_on( bench );
}



fn send_tokio_tp()
{
	let bench = async move
	{
		let     sum  = Sum(5)                   ;
		let     mb   = Inbox::new()             ;
		let mut addr = Addr::new( mb.sender() ) ;
		let mut pool = TokioTp::new()           ;

		// This is ugly right now. It will be more ergonomic in the future.
		//
		pool.spawn( mb.start_fut( sum ) ).expect( "Spawning failed" );

		for _i in 0..100usize
		{
			addr.send( Add( 10 ) ).await.expect( "Send failed" );
		}

		let res = addr.call( Show{} ).await.expect( "Call failed" );
		assert_eq!( 1005, res );
	};

	block_on( bench );
}



fn call_tokio_tp()
{
	let bench = async move
	{
		let     sum  = Sum(5)                   ;
		let     mb   = Inbox::new()             ;
		let mut addr = Addr::new( mb.sender() ) ;
		let mut pool = TokioTp::new()           ;

		// This is ugly right now. It will be more ergonomic in the future.
		//
		pool.spawn( mb.start_fut( sum ) ).expect( "Spawning failed" );

		for _i in 0..100usize
		{
			addr.call( Add( 10 ) ).await.expect( "Send failed" );
		}

		let res = addr.call( Show{} ).await.expect( "Call failed" );
		assert_eq!( 1005, res );
	};

	block_on( bench );
}



fn send_juliex()
{
	let bench = async move
	{
		let     sum  = Sum(5)                   ;
		let     mb   = Inbox::new()             ;
		let mut addr = Addr::new( mb.sender() ) ;
		let mut pool = Juliex::new()            ;

		// This is ugly right now. It will be more ergonomic in the future.
		//
		pool.spawn( mb.start_fut( sum ) ).expect( "Spawning failed" );

		for _i in 0..100usize
		{
			addr.send( Add( 10 ) ).await.expect( "Send failed" );
		}

		let res = addr.call( Show{} ).await.expect( "Call failed" );
		assert_eq!( 1005, res );
	};

	block_on( bench );
}



fn call_juliex()
{
	let bench = async move
	{
		let     sum  = Sum(5)                   ;
		let     mb   = Inbox::new()             ;
		let mut addr = Addr::new( mb.sender() ) ;
		let mut pool = Juliex::new()            ;

		// This is ugly right now. It will be more ergonomic in the future.
		//
		pool.spawn( mb.start_fut( sum ) ).expect( "Spawning failed" );

		for _i in 0..100usize
		{
			addr.call( Add( 10 ) ).await.expect( "Send failed" );
		}

		let res = addr.call( Show{} ).await.expect( "Call failed" );
		assert_eq!( 1005, res );
	};

	block_on( bench );
}




struct Accu( AtomicU64 );

impl Accu
{
	#[ inline( never ) ]
	//
	async fn add( &self, v: Add )
	{
		self.0.fetch_add( v.0, Ordering::Relaxed );
	}

	async fn show( &self ) -> u64
	{
		self.0.load( Ordering::Relaxed )
	}
}



fn method()
{
	block_on( async
	{
		let sum  = Arc::new( Accu( AtomicU64::from( 5 ) ) );
		let sum2 = sum.clone();

		thread::spawn( move ||
		{
			let mut thread_pool = LocalPool::new();

			let thread_program = async move
			{
				for _i in 0..100usize
				{
					sum2.add( Add( 10 ) ).await;
				}

				let res = sum2.show().await;
				assert_eq!( 1005, res );
			};

			thread_pool.run_until( thread_program );
		});
	})
}



// fn actix_dosend()
// {
// 	actix::System::run( ||
// 	{
// 		Arbiter::spawn( async
// 		{
// 			let sum  = AxSum(5)    ;
// 			let addr = sum.start() ;

// 			thread::spawn( move ||
// 			{
// 				Arbiter::spawn( async move
// 				{
// 					for _i in 0..100usize
// 					{
// 						addr.do_send( AxAdd( 10 ) );
// 					}

// 					let res = addr.send( AxShow{} ).await.unwrap();

// 					assert_eq!( 1005, res );

// 					Ok(())

// 				}.boxed().compat());

// 			});



// 			actix::System::current().stop();

// 			Ok(())

// 		}.boxed().compat());

// 	});
// }


// fn actix_send()
// {
// 	actix::System::run( ||
// 	{
// 		Arbiter::spawn( async
// 		{
// 			let sum  = AxSum(5)    ;
// 			let addr = sum.start() ;

// 			thread::spawn( move ||
// 			{
// 				Arbiter::spawn( async move
// 				{
// 					for _i in 0..100usize
// 					{
// 						addr.send( AxAdd( 10 ) ).await.expect( "failed sending actix message" );
// 					}

// 					let res = addr.send( AxShow{} ).await.unwrap();

// 					assert_eq!( 1006, res );

// 					Ok(())

// 				}.boxed().compat());

// 			});



// 			actix::System::current().stop();

// 			Ok(())

// 		}.boxed().compat());

// 	});
// }

// struct AxSum (u64);
// struct AxAdd (u64);
// struct AxShow     ;

// impl AxMessage for AxAdd  { type Result  = ()              ; }
// impl AxMessage for AxShow { type Result  = u64             ; }
// impl AxActor   for AxSum  { type Context = AxContext<Self> ; }


// impl AxHandler< AxAdd > for AxSum
// {
// 	type Result  = ()
// ;
// 	fn handle( &mut self, msg: AxAdd, _ctx: &mut AxContext<Self> )
// 	{
// 		self.0 += msg.0;
// 	}
// }


// impl AxHandler< AxShow > for AxSum
// {
// 	type Result = u64
// ;
// 	fn handle( &mut self, _msg: AxShow, _ctx: &mut AxContext<Self> ) -> Self::Result
// 	{
// 		self.0
// 	}
// }




fn bench_calls( c: &mut Criterion )
{
	c.bench
	(
		"Multi Thread Delivery",

		Benchmark::new   ( "Send ThreadPool x100"    , |b| b.iter( || send_threadpool() ) )
			.with_function( "Call ThreadPool x100"    , |b| b.iter( || call_threadpool() ) )
			.with_function( "Send TokioTp    x100"    , |b| b.iter( || send_tokio_tp  () ) )
			.with_function( "Call TokioTp    x100"    , |b| b.iter( || call_tokio_tp  () ) )
			.with_function( "Send Juliex     x100"    , |b| b.iter( || send_juliex    () ) )
			.with_function( "Call Juliex     x100"    , |b| b.iter( || call_juliex    () ) )
			.with_function( "async method x100"       , |b| b.iter( || method       () ) )
			// .with_function( "actix do_send x100"      , |b| b.iter( || actix_dosend () ) )
			// .with_function( "actix send x100"         , |b| b.iter( || actix_send   () ) )
	);
}

criterion_group!( benches, bench_calls );
criterion_main! ( benches              );
