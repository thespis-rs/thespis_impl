use
{
	async_executors   :: { *                                                     } ,
	criterion         :: { Criterion, criterion_group, criterion_main, BatchSize } ,
	futures           :: { executor::{ block_on }                                } ,
	thespis           :: { *                                                     } ,
	thespis_impl      :: { *                                                     } ,
	std               :: { convert::TryFrom                                      } ,
	tokio             :: { runtime::Builder                                      } ,
	actix             :: { Actor as _, ActorFuture                               } ,
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


struct Accu( u64 );

impl Accu
{
	#[ inline( never ) ]
	//
	async fn add( &mut self, v: Add )
	{
		self.0 += v.0;
	}

	async fn show( &self ) -> u64
	{
		self.0
	}
}



fn seq( c: &mut Criterion )
{
	// let _ = flexi_logger::Logger::with_str( "trace" ).start();


	let mut group = c.benchmark_group( "seq" );

	for msgs in [ 1, 100, 10000 ].iter()
	{
		// match buffer_size
		// {
		// 	10  => { group.sample_size( 100 ); }
		// 	100 => { group.sample_size( 50  ); }
		// 	200 => { group.sample_size( 30  ); }
		// 	_   => { unreachable!();           }
		// }

		group.sample_size( 30 );

		group.bench_function
		(
			format!( "send: {} msgs", &msgs ),

			|b| b.iter_batched
			(
				move || // setup
				{
					let (sum_in_addr, sum_in_mb) = Addr::builder().bounded( Some(BOUNDED) ).build() ;
					let sum      = Sum{ total: 5, inner: sum_in_addr }                              ;
					let (sum_addr, sum_mb) = Addr::builder().bounded( Some(BOUNDED) ).build()       ;

					let exec = TokioCt::try_from( &mut Builder::new() ).expect( "build runtime" );
					let sumin = SumIn{ count: 0 };

					let sumin_handle = exec.spawn_handle( sum_in_mb.start_fut( sumin ) ).expect( "spawn" );
					let sum_handle   = exec.spawn_handle( sum_mb   .start_fut( sum   ) ).expect( "spawn" );

					(sum_addr, sumin_handle, sum_handle, exec)
				},


				|(mut sum_addr, sumin_handle, sum_handle, exec)| // measure
				{
					exec.block_on( async move
					{
						for _ in 0..*msgs
						{
							sum_addr.send( Add(10) ).await.expect( "Send failed" );
						}

						let res = sum_addr.call( Show{} ).await.expect( "Call failed" );

						assert_eq!( *msgs as u64 *10 + 5 + termial( *msgs as u64 ), res );

						drop( sum_addr );

						sumin_handle.await;
						sum_handle  .await;
					});
				},

				BatchSize::SmallInput
			)
		);


		// Currently doesn't work, as it won't wait until the message is processed, so the
		// assert comes to early. We would have to synchronize somehow.
		//
		// group.bench_function
		// (
		// 	format!( "actix do_send: {} msgs", &msgs ),

		// 	|b| b.iter
		// 	(
		// 		||
		// 		{
		// 			actix_rt::System::new( "main" ).block_on( async move
		// 			{
		// 				let sum_in      = SumIn{ count: 0 };
		// 				let sum_in_addr = SumIn::start( sum_in );

		// 				let sum      = ActixSum{ total: 5, inner: sum_in_addr };
		// 				let sum_addr = ActixSum::start( sum );

		// 				for _ in 0..*msgs
		// 				{
		// 					sum_addr.do_send( Add(10) );
		// 				}

		// 				let res = sum_addr.send( Show{} ).await.expect( "Call failed" );

		// 				assert_eq!( *msgs as u64 *10 + 5 + termial( *msgs as u64 ), res );

		// 				actix_rt::System::current().stop();
		// 			});
		// 		}
		// 	)
		// );


		group.bench_function
		(
			format!( "call: {} msgs", &msgs ),

			|b| b.iter_batched
			(
				move || // setup
				{
					let (sum_in_addr, sum_in_mb) = Addr::builder().bounded( Some(BOUNDED) ).build() ;
					let sum      = Sum{ total: 5, inner: sum_in_addr }                              ;
					let (sum_addr, sum_mb) = Addr::builder().bounded( Some(BOUNDED) ).build()       ;

					let exec = TokioCt::try_from( &mut Builder::new() ).expect( "build runtime" );
					let sumin = SumIn{ count: 0 };

					let sumin_handle = exec.spawn_handle( sum_in_mb.start_fut( sumin ) ).expect( "spawn" );
					let sum_handle   = exec.spawn_handle( sum_mb   .start_fut( sum   ) ).expect( "spawn" );

					(sum_addr, sumin_handle, sum_handle, exec)
				},


				|(mut sum_addr, sumin_handle, sum_handle, exec)| // measure
				{
					exec.block_on( async move
					{
						for _ in 0..*msgs
						{
							sum_addr.call( Add(10) ).await.expect( "Send failed" );
						}

						let res = sum_addr.call( Show{} ).await.expect( "Call failed" );

						assert_eq!( *msgs as u64 *10 + 5 + termial( *msgs as u64 ), res );

						drop( sum_addr );

						sumin_handle.await;
						sum_handle  .await;
					});
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
						let sum_in      = SumIn{ count: 0 };
						let sum_in_addr = SumIn::start( sum_in );

						let sum      = ActixSum{ total: 5, inner: sum_in_addr };
						let sum_addr = ActixSum::start( sum );

						for _ in 0..*msgs
						{
							sum_addr.send( Add(10) ).await.expect( "Send failed" );
						}

						let res = sum_addr.send( Show{} ).await.expect( "Call failed" );

						assert_eq!( *msgs as u64 *10 + 5 + termial( *msgs as u64 ), res );

						actix_rt::System::current().stop();
					});
				}
			)
		);





		group.bench_function
		(
			format!( "async method: {} msgs", &msgs ),

			|b| b.iter
			(
				||
				{
					let res = block_on( async
					{
						let mut sum  = Accu( 5 );

						for _i in 0..*msgs
						{
							sum.add( Add( 10 ) ).await;
						}

						sum.show().await
					});

					assert_eq!( msgs*10 + 5, res );
				}
			)
		);
	}
}


criterion_group!( benches, seq );
criterion_main! ( benches      );
