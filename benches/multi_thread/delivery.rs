use
{
	async_chanx       :: { * } ,
	criterion         :: { Criterion, criterion_group, criterion_main, BatchSize           } ,
	futures           :: { executor::{ block_on }, channel::oneshot } ,
	thespis           :: { *                                                               } ,
	thespis_impl      :: { *                                                               } ,
	std               :: { thread, sync::{ Arc, atomic::{ AtomicU64, Ordering } }          } ,
	tokio             :: { sync::mpsc                                                      } ,
	actix             :: { Actor as _, ActorFuture } ,
};


const BOUNDED: usize = 11;


fn termial( n: u64 ) -> u64
{
	n * ( n + 1 ) / 2
}


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



struct ActixSum
{
	pub total: u64,
	pub inner: actix::Addr<SumIn>,
}

impl actix::Actor for ActixSum { type Context = actix::Context<Self>; }
impl actix::Actor for SumIn    { type Context = actix::Context<Self>; }

impl actix::Message for Add  { type Result = () ; }
impl actix::Message for Show { type Result = u64; }


impl actix::Handler< Add > for ActixSum
{
	type Result = actix::ResponseActFuture< Self, <Add as actix::Message>::Result >;

	fn handle( &mut self, msg: Add, _ctx: &mut Self::Context ) -> Self::Result
	{
		let action = self.inner.send( Show );

		let act_fut = actix::fut::wrap_future::<_, Self>(action);

		let update_self = act_fut.map( move |result, actor, _ctx|
		{
			actor.total += msg.0 + result.expect( "Call SumIn" );
		});

		Box::pin( update_self )
	}
}


impl actix::Handler< Show > for ActixSum
{
	type Result = u64;

	fn handle( &mut self, _msg: Show, _ctx: &mut actix::Context<Self> ) -> Self::Result
	{
		self.total
	}
}


impl actix::Handler< Show > for SumIn
{
	type Result = u64;

	fn handle( &mut self, _msg: Show, _ctx: &mut actix::Context<Self> ) -> Self::Result
	{
		self.count += 1;
		self.count
	}
}



// fn send_threadpool()
// {
// 	let bench = async move
// 	{
// 		let (tx, rx) = mpsc::channel( BOUNDED )                                            ;
// 		let     sum  = Sum(5)                                                              ;
// 		let     mb   = Inbox::new( None, Box::new( rx ) )                                  ;
// 		let mut addr = Addr::new( mb.id(), mb.name(), Box::new( TokioSender::new( tx ) ) ) ;
// 		let     pool = AsyncStd::default()                                                 ;

// 		// This is ugly right now. It will be more ergonomic in the future.
// 		//
// 		let handle = pool.spawn( mb.start_fut( sum ) ).expect( "Spawning failed" );

// 		let sender_thread = thread::spawn( move ||
// 		{
// 			let thread_program = async move
// 			{
// 				for _i in 0..100usize
// 				{
// 					addr.send( Add( 10 ) ).await.expect( "Send failed" );
// 				}

// 				let res = addr.call( Show{} ).await.expect( "Call failed" );
// 				assert_eq!( 1005, res );
// 			};

// 			block_on( thread_program );
// 		});

// 		sender_thread.join().expect( "join thread" );
// 		handle.await().expect( "join mb" );
// 	};

// 	block_on( bench );
// }



// fn call_threadpool()
// {
// 	let bench = async move
// 	{
// 		let     sum  = Sum(5)                                          ;
// 		let     mb   = Inbox::new()                                    ;
// 		let mut addr = Addr::new( mb.sender() )                        ;
// 		let     pool = ThreadPool::new().expect( "create threadpool" ) ;

// 		// This is ugly right now. It will be more ergonomic in the future.
// 		//
// 		pool.spawn( mb.start_fut( sum ) ).expect( "Spawning failed" );

// 		let sender_thread = thread::spawn( move ||
// 		{
// 			let thread_program = async move
// 			{
// 				for _i in 0..100usize
// 				{
// 					addr.call( Add( 10 ) ).await.expect( "Call failed" );
// 				}

// 				let res = addr.call( Show{} ).await.expect( "Call failed" );
// 				assert_eq!( 1005, res );
// 			};

// 			block_on( thread_program );
// 		});

// 		sender_thread.join().expect( "join thread" );
// 	};

// 	block_on( bench );
// }




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





fn spsc( c: &mut Criterion )
{
	// let _ = flexi_logger::Logger::with_str( "trace" ).start();


	let mut group = c.benchmark_group( "Spsc" );

	for msgs in [ 1, 100, 10000 ].iter()
	{
		// match buffer_size
		// {
		// 	10  => { group.sample_size( 100 ); }
		// 	100 => { group.sample_size( 50  ); }
		// 	200 => { group.sample_size( 30  ); }
		// 	_   => { unreachable!();           }
		// }

		group.sample_size( 10 );

		group.bench_function
		(
			format!( "send: {} msgs", &msgs ),

			|b| b.iter_batched
			(
				move || // setup
				{
					let (tx, rx)    = mpsc::channel( BOUNDED )                                                        ;
					let sum_in_mb   = Inbox::new( None, Box::new( rx ) )                                              ;
					let tx          = Box::new( TokioSender::new( tx ).sink_map_err( |e| Box::new(e) as SinkError ) ) ;
					let sum_in_addr = Addr::new( sum_in_mb.id(), sum_in_mb.name(), tx ) ;

					let (tx, rx) = mpsc::channel( BOUNDED )                                                    ;
					let sum_mb   = Inbox::new( None, Box::new( rx ) )                                          ;
					let tx       = Box::new( TokioSender::new( tx ).sink_map_err( |e| Box::new(e) as SinkError ) ) ;
					let sum_addr = Addr::new( sum_mb.id(), sum_mb.name(), tx ) ;
					let sum      = Sum{ total: 5, inner: sum_in_addr }                                         ;

					let sumin_thread = thread::spawn( move ||
					{
						let sum_in = SumIn{ count: 0 };

						async_std::task::block_on( sum_in_mb.start_fut( sum_in ) );
					});

					let sum_thread = thread::spawn( move ||
					{
						async_std::task::block_on( sum_mb.start_fut( sum ) );
					});

					(sum_addr, sumin_thread, sum_thread)
				},


				|(mut sum_addr, sumin_thread, sum_thread)| // measure
				{
					async_std::task::block_on( async move
					{
						for _ in 0..*msgs
						{
							sum_addr.send( Add(10) ).await.expect( "Send failed" );
						}

						let res = sum_addr.call( Show{} ).await.expect( "Call failed" );

						assert_eq!( *msgs as u64 *10 + 5 + termial( *msgs as u64 ), res );
					});

					sumin_thread.join().expect( "join sum_in thread" );
					sum_thread  .join().expect( "join sum    thread" );
				},

				BatchSize::SmallInput
			)
		);


		group.bench_function
		(
			format!( "actix send: {} msgs", &msgs ),

			|b| b.iter
			(
				||
				{
					actix_rt::System::new( "main" ).block_on( async move
					{
						let mut sum_in_thread = actix::Arbiter::new();
						let mut sum_thread    = actix::Arbiter::new();

						let sum_in      = SumIn{ count: 0 };
						let sum_in_addr = SumIn::start_in_arbiter( &sum_in_thread, |_| sum_in );

						let sum      = ActixSum{ total: 5, inner: sum_in_addr };
						let sum_addr = ActixSum::start_in_arbiter( &sum_thread, |_| sum );

						for _ in 0..*msgs
						{
							sum_addr.send( Add(10) ).await.expect( "Send failed" );
						}

						let res = sum_addr.send( Show{} ).await.expect( "Call failed" );

						assert_eq!( *msgs as u64 *10 + 5 + termial( *msgs as u64 ), res );

						sum_in_thread.stop();
						sum_thread   .stop();

						sum_in_thread.join().expect( "join arbiter thread" );
						sum_thread   .join().expect( "join arbiter thread" );

						actix_rt::System::current().stop();
					});
				}
			)
		);


		group.bench_function
		(
			format!( "call: {} msgs", &msgs ),

			|b| b.iter_batched
			(
				move || // setup
				{
					let (tx, rx)    = mpsc::channel( BOUNDED )                                                        ;
					let sum_in_mb   = Inbox::new( None, Box::new( rx ) )                                              ;
					let tx          = Box::new( TokioSender::new( tx ).sink_map_err( |e| Box::new(e) as SinkError ) ) ;
					let sum_in_addr = Addr::new( sum_in_mb.id(), sum_in_mb.name(), tx )                               ;

					let (tx, rx) = mpsc::channel( BOUNDED )                                                           ;
					let sum_mb   = Inbox::new( None, Box::new( rx ) )                                                 ;
					let tx          = Box::new( TokioSender::new( tx ).sink_map_err( |e| Box::new(e) as SinkError ) ) ;
					let sum_addr = Addr::new( sum_mb.id(), sum_mb.name(), tx )                                        ;
					let sum      = Sum{ total: 5, inner: sum_in_addr }                                                ;

					let sumin_thread = thread::spawn( move ||
					{
						let sum_in = SumIn{ count: 0 };

						async_std::task::block_on( sum_in_mb.start_fut( sum_in ) );
					});

					let sum_thread = thread::spawn( move ||
					{
						async_std::task::block_on( sum_mb.start_fut( sum ) );
					});

					(sum_addr, sumin_thread, sum_thread)
				},


				|(mut sum_addr, sumin_thread, sum_thread)| // measure
				{
					async_std::task::block_on( async move
					{
						for _ in 0..*msgs
						{
							sum_addr.call( Add(10) ).await.expect( "Send failed" );
						}

						let res = sum_addr.call( Show{} ).await.expect( "Call failed" );

						assert_eq!( *msgs as u64 *10 + 5 + termial( *msgs as u64 ), res );
					});

					sumin_thread.join().expect( "join sum_in thread" );
					sum_thread  .join().expect( "join sum    thread" );
				},

				BatchSize::SmallInput
			)
		);





		group.bench_function
		(
			format!( "async method: {} msgs", &msgs ),

			|b| b.iter_batched
			(
				move || // setup
				{
					let (start_tx, start_rx) = oneshot::channel();

					let sum  = Arc::new( Accu( AtomicU64::from( 5 ) ) );
					let sum2 = sum.clone();

					let sender_thread = thread::spawn( move ||
					{
						block_on( async move
						{
							start_rx.await.expect( "wait start_rx" );

							for _i in 0..*msgs
							{
								sum.add( Add( 10 ) ).await;
							}
						});
					});

					(start_tx, sum2, sender_thread)
				},


				|(start_tx, sum2, sender_thread)| // routine
				{
					start_tx.send(()).expect( "oneshot send" );

					sender_thread.join().expect( "join thread" );

					let res = block_on( sum2.show() );
					assert_eq!( msgs*10 + 5, res );
				},

				BatchSize::SmallInput
			)
		);
	}
}


criterion_group!( benches, spsc );
criterion_main! ( benches       );
