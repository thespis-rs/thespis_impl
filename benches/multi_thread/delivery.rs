#![ feature( await_macro, async_await, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]


use
{
	criterion         :: { Criterion, Benchmark, criterion_group, criterion_main  } ,
	futures           :: { executor::{ block_on }, executor::{ LocalPool }        } ,
	thespis           :: { *                                                      } ,
	thespis_impl      :: { *, runtime::rt                                         } ,
	std               :: { thread, sync::{ Arc, atomic::{ AtomicU64, Ordering } } } ,
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



fn send()
{
	let bench = async move
	{
		let     sum  = Sum(5);
		let mut addr = Addr::try_from( sum ).expect( "Failed to create address" );


		thread::spawn( move ||
		{
			let thread_program = async move
			{
				for _i in 0..100usize
				{
					await!( addr.send( Add( 10 ) ) ).expect( "Send failed" );
				}

				let res = await!( addr.call( Show{} ) ).expect( "Call failed" );
				assert_eq!( 1005, res );
			};

			rt::spawn( thread_program ).expect( "spawn thread_program" );
			rt::run();
		});
	};

	rt::spawn( bench ).expect( "spawn send bench" );
	rt::run();
}



fn call()
{
	let bench = async move
	{
		let     sum  = Sum(5);
		let mut addr = Addr::try_from( sum ).expect( "Failed to create address" );


		thread::spawn( move ||
		{
			let thread_program = async move
			{
				for _i in 0..100usize
				{
					await!( addr.call( Add( 10 ) ) ).expect( "Send failed" );
				}

				let res = await!( addr.call( Show{} ) ).expect( "Call failed" );
				assert_eq!( 1005, res );
			};

			rt::spawn( thread_program ).expect( "spawn thread_program" );
			rt::run();
		});
	};

	rt::spawn( bench ).expect( "spawn call bench" );
	rt::run();
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
					await!( sum2.add( Add( 10 ) ) );
				}

				let res = await!( sum2.show() );
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

// 					let res = await!( addr.send( AxShow{} ) ).unwrap();

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
// 						await!( addr.send( AxAdd( 10 ) ) ).expect( "failed sending actix message" );
// 					}

// 					let res = await!( addr.send( AxShow{} ) ).unwrap();

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
// 	type Result  = u64
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

		Benchmark::new   ( "Send x100"               , |b| b.iter( || send         () ) )
			.with_function( "Call x100"               , |b| b.iter( || call         () ) )
			.with_function( "async method x100"       , |b| b.iter( || method       () ) )
			// .with_function( "actix do_send x100"      , |b| b.iter( || actix_dosend () ) )
			// .with_function( "actix send x100"         , |b| b.iter( || actix_send   () ) )
	);
}

criterion_group!( benches, bench_calls );
criterion_main! ( benches              );
