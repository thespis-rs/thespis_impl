use
{
	async_chanx       :: { * } ,
	async_executors   :: { * } ,
	criterion         :: { Criterion, criterion_group, criterion_main, BatchSize           } ,
	futures           :: { executor::{ block_on }, channel::oneshot } ,
	thespis           :: { *                                                               } ,
	thespis_impl      :: { *                                                               } ,
	std               :: { thread, sync::{ Arc, atomic::{ AtomicU64, Ordering } }          } ,
	tokio             :: { sync::mpsc                                                      } ,
	actix             :: { Actor as _, ActorFuture } ,
};


const BOUNDED: usize = 11;


#[ derive( Actor ) ] struct Sum( u64 );

struct Add ( u64 );
struct Show       ;

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


impl actix::Message for Add  { type Result  = ();  }
impl actix::Message for Show { type Result  = u64; }

impl actix::Actor   for Sum
{
	type Context = actix::Context<Self>;

	fn started( &mut self, ctx: &mut Self::Context )
	{
		ctx.set_mailbox_capacity( BOUNDED );
	}
}


impl actix::Handler< Add > for Sum
{
	type Result = actix::ResponseActFuture<Self, ()>;

	fn handle( &mut self, msg: Add, _ctx: &mut actix::Context<Self> ) -> Self::Result
	{
		let action = async move
		{
			msg.0
		};

		let act_fut = actix::fut::wrap_future::<_, Self>(action);

		let update_self = act_fut.map( |result, actor, _ctx|
		{
			actor.0 += result;
		});

		Box::pin( update_self )
	}
}


impl actix::Handler< Show > for Sum
{
	type Result = u64;

	fn handle( &mut self, _msg: Show, _ctx: &mut actix::Context<Self> ) -> Self::Result
	{
		self.0
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
					let (tx, rx) = mpsc::channel( BOUNDED )                                            ;
					let     mb   = Inbox::new( None, Box::new( rx ) )                                  ;
					let mut addr = Addr::new( mb.id(), mb.name(), Box::new( TokioSender::new( tx ) ) ) ;
					let     sum  = Sum(5);

					let (start_tx, start_rx) = oneshot::channel();

					let sender_thread = thread::spawn( move ||
					{
						block_on( async move
						{
							start_rx.await.expect( "wait start_rx" );

							for _i in 0..*msgs
							{
								addr.send( Add( 10 ) ).await.expect( "Send failed" );
							}

							let res = addr.call( Show{} ).await.expect( "Call failed" );
							assert_eq!( msgs*10 + 5, res );
						});
					});

					let mb_handle = mb.start_fut( sum );

					(start_tx, mb_handle, sender_thread)
				},


				|(start_tx, mb_handle, sender_thread)| // measure
				{
					start_tx.send(()).expect( "oneshot send" );

					block_on( mb_handle );

					sender_thread.join().expect( "join thread" );
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
					actix_rt::System::new("main").block_on( async move
					{
						let (start_tx, start_rx) = oneshot::channel();

						actix::Arbiter::spawn( async move
						{
							let sum  = Sum(5);
							let addr = sum.start();

							if start_tx.send(addr).is_err()
							{
								panic!( "send on oneshot" );
							}
						});

						let addr: actix::Addr<Sum> = start_rx.await.expect( "wait start_rx" );

						for _i in 0..*msgs
						{
							addr.send( Add( 10 ) ).await.expect( "Send failed" );
						}

						let res = addr.send( Show{} ).await.expect( "Call failed" );
						assert_eq!( msgs*10 + 5, res );

						actix::System::current().stop();
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
					let (tx, rx) = mpsc::channel( BOUNDED )                                            ;
					let     mb   = Inbox::new( None, Box::new( rx ) )                                  ;
					let mut addr = Addr::new( mb.id(), mb.name(), Box::new( TokioSender::new( tx ) ) ) ;
					let     sum  = Sum(5);

					let (start_tx, start_rx) = oneshot::channel();

					let sender_thread = thread::spawn( move ||
					{
						block_on( async move
						{
							start_rx.await.expect( "wait start_rx" );

							for _i in 0..*msgs
							{
								addr.call( Add( 10 ) ).await.expect( "Send failed" );
							}

							let res = addr.call( Show{} ).await.expect( "Call failed" );
							assert_eq!( msgs*10 + 5, res );
						});
					});

					let mb_handle = mb.start_fut( sum );

					(start_tx, mb_handle, sender_thread)
				},


				|(start_tx, mb_handle, sender_thread)| // routine
				{
					start_tx.send(()).expect( "oneshot send" );

					block_on( mb_handle );

					sender_thread.join().expect( "join thread" );
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
