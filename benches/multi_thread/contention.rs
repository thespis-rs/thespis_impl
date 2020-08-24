use
{
	criterion         :: { Criterion, criterion_group, criterion_main } ,
	futures           :: { FutureExt                                  } ,
	thespis           :: { *                                          } ,
	thespis_impl      :: { *                                          } ,
	std               :: { thread, sync::{ Arc, Mutex }               } ,
	actix             :: { Actor as _, ActorFuture                    } ,
	async_std         :: { task::block_on                             } ,
};


const BOUNDED: usize = 16;
const SENDERS: usize = 10;


fn termial( n: usize ) -> usize
{
	n * (n + 1) / 2
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
	#[async_fn] fn handle( &mut self, msg: Add )
	{
		let inner = self.inner.call( Show ).await.expect( "call inner" );

		self.total += msg.0 + inner;
	}
}


impl Handler< Show > for Sum
{
	#[async_fn] fn handle( &mut self, _msg: Show ) -> u64
	{
		self.total
	}
}


impl Handler< Show > for SumIn
{
	#[async_fn] fn handle( &mut self, _msg: Show ) -> u64
	{
		self.count += 1;
		self.count
	}
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


struct Accu
{
	count: Mutex<u64>    ,
	inner: Mutex<AccuIn> ,
}

impl Accu
{
	#[ inline( never ) ]
	//
	async fn add( &self, v: Add )
	{
		let from_in = self.inner.lock().unwrap().show().await;
		let mut count = self.count.lock().unwrap();
		*count += v.0 + from_in;
	}

	#[ inline( never ) ]
	//
	async fn show( &self ) -> u64
	{
		*self.count.lock().unwrap()
	}
}


struct AccuIn( u64 );

impl AccuIn
{
	#[ inline( never ) ]
	//
	async fn show( &mut self ) -> u64
	{
		self.0 += 1;
		self.0
	}
}



fn mpsc( c: &mut Criterion )
{
	// let _ = flexi_logger::Logger::with_str( "trace" ).start();


	let mut group = c.benchmark_group( "Mpsc" );

	for msgs in [ 10, 1000 ].iter()
	{
		let total_msgs = SENDERS * *msgs *10 + 5 + termial( SENDERS * *msgs );

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

			|b| b.iter
			(
				||
				{
					let (sum_in_addr , sum_in_mb) = Addr::builder().bounded( Some(BOUNDED) ).build() ;
					let (mut sum_addr, sum_mb   ) = Addr::builder().bounded( Some(BOUNDED) ).build() ;

					let sum    = Sum   { total: 5, inner: sum_in_addr } ;
					let sum_in = SumIn { count: 0 }                     ;

					let mut senders = Vec::with_capacity( SENDERS );

					for _ in 0..SENDERS
					{
						let mut sum_addr2 = sum_addr.clone();

						senders.push( thread::spawn( move ||
						{
							block_on( async move
							{
								for _ in 0..*msgs
								{
									sum_addr2.send( Add(10) ).await.expect( "Send failed" );
								}
							});
						}));
					}

					let sum_in_thread = thread::spawn( move || block_on( sum_in_mb.start( sum_in ) ) );
					let sum_thread    = thread::spawn( move || block_on( sum_mb   .start( sum    ) ) );

					for sender in senders.into_iter()
					{
						sender.join().expect( "join sender thread" );
					}

					block_on( async move
					{
						let res = sum_addr.call( Show{} ).await.expect( "Call failed" );

						assert_eq!( total_msgs, res as usize );
					});

					sum_in_thread.join().expect( "join sum_in thread" );
					sum_thread   .join().expect( "join sum    thread" );
				}
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

						let mut senders = Vec::with_capacity( SENDERS );

						for _ in 0..SENDERS
						{
							let addr = sum_addr.clone();
							let arb  = actix::Arbiter::new();
							let arb2 = arb.clone();

							let fut = async move
							{
								for _ in 0..*msgs
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

						assert_eq!( total_msgs, res as usize );

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

			|b| b.iter
			(
				||
				{
					let (sum_in_addr , sum_in_mb) = Addr::builder().bounded( Some(BOUNDED) ).build() ;
					let (mut sum_addr, sum_mb   ) = Addr::builder().bounded( Some(BOUNDED) ).build() ;

					let sum    = Sum   { total: 5, inner: sum_in_addr } ;
					let sum_in = SumIn { count: 0 }                     ;

					let mut senders = Vec::with_capacity( SENDERS );

					for _ in 0..SENDERS
					{
						let mut sum_addr2 = sum_addr.clone();

						senders.push( thread::spawn( move ||
						{
							block_on( async move
							{
								for _ in 0..*msgs
								{
									sum_addr2.call( Add(10) ).await.expect( "Send failed" );
								}
							});
						}));
					}

					let sum_in_thread = thread::spawn( move || block_on( sum_in_mb.start( sum_in ) ) );
					let sum_thread    = thread::spawn( move || block_on( sum_mb   .start( sum    ) ) );

					for sender in senders.into_iter()
					{
						sender.join().expect( "join sender thread" );
					}

					block_on( async move
					{
						let res = sum_addr.call( Show{} ).await.expect( "Call failed" );

						assert_eq!( total_msgs, res as usize );
					});

					sum_in_thread.join().expect( "join sum_in thread" );
					sum_thread   .join().expect( "join sum    thread" );
				}
			)
		);





		group.bench_function
		(
			format!( "async method: {} msgs", &msgs ),

			|b| b.iter
			(
				move || // setup
				{
					let sum  = Arc::new( Accu
					{
						count: Mutex::new( 5 ),
						inner: Mutex::new( AccuIn(0) )
					});

					let mut senders  = Vec::with_capacity( SENDERS );

					for _ in 0..SENDERS
					{
						let accu = sum.clone();

						senders.push( thread::spawn( move ||
						{
							block_on( async move
							{
								for _i in 0..*msgs
								{
									accu.add( Add( 10 ) ).await;
								}
							});
						}));
					}


					for sender in senders.into_iter()
					{
						sender.join().expect( "join thread" );
					}

					let res = block_on( sum.show() );

					assert_eq!( total_msgs, res as usize );
				}
			)
		);
	}
}


criterion_group!( benches, mpsc );
criterion_main! ( benches       );
