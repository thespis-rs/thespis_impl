#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

use
{
	criterion         :: { Criterion, Benchmark, criterion_group, criterion_main } ,
	futures           :: { future::{ FutureExt, TryFutureExt }, executor::{ block_on }, executor::{ LocalPool }, task::LocalSpawnExt } ,
	thespis           :: { *                                                     } ,
	thespis_impl      :: { single_thread::*                                                     } ,
	actix             :: { Actor as AxActor, Message as AxMessage, Handler as AxHandler, Context as AxContext, Arbiter } ,
	tokio_async_await :: { await                                       },

};



#[ derive( Actor ) ] struct Sum( u64 );

struct Add (u64);
struct Show     ;

impl Message for Add  { type Result = () ; }
impl Message for Show { type Result = u64; }


impl Handler< Add > for Sum
{
	fn handle( &mut self, msg: Add ) -> Response<Add> { async move
	{

		self.0 += msg.0;

	}.boxed() }
}


impl Handler< Show > for Sum
{
	fn handle( &mut self, _msg: Show ) -> Response<Show> { async move
	{

		self.0

	}.boxed() }
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

	#[ inline( always ) ]
	//
	async fn add_inline( &mut self, v: Add )
	{
		self.0 += v.0;
	}


	async fn show( &mut self ) -> u64
	{
		self.0
	}
}


fn send()
{
	let mut pool  = LocalPool::new();
	let mut exec  = pool.spawner();
	let mut exec2 = exec.clone();

	let bench = async move
	{
		let     sum              = Sum(5)                  ;
		let     mb  : Inbox<Sum> = Inbox::new()            ;
		let mut addr             = Addr::new( mb.sender() );

		// This is ugly right now. It will be more ergonomic in the future.
		//
		let move_mb = async move { await!( mb.start( sum ) ); };
		exec2.spawn_local( move_mb ).expect( "Spawning failed" );

		for _i in 0..100usize
		{
			await!( addr.send( Add( 10 ) ) );
		}

		let res = await!( addr.call( Show{} ) );
		assert_eq!( 1005, res );
	};

	exec.spawn_local( bench ).expect( "Spawn benchmark" );

	pool.run();
}


fn call()
{
	let mut pool  = LocalPool::new();
	let mut exec  = pool.spawner();
	let mut exec2 = exec.clone();

	let bench = async move
	{
		let     sum              = Sum(5)                  ;
		let     mb  : Inbox<Sum> = Inbox::new()            ;
		let mut addr             = Addr::new( mb.sender() );

		// This is ugly right now. It will be more ergonomic in the future.
		//
		let move_mb = async move { await!( mb.start( sum ) ); };
		exec2.spawn_local( move_mb ).expect( "Spawning failed" );

		for _i in 0..100usize
		{
			await!( addr.call( Add( 10 ) ) );
		}

		let res = await!( addr.call( Show{} ) );
		assert_eq!( 1005, res );
	};

	exec.spawn_local( bench ).expect( "Spawn benchmark" );

	pool.run();
}


fn actix_dosend()
{
	actix::System::run( ||
	{
		Arbiter::spawn( async
		{
			let sum  = AxSum(5)    ;
			let addr = sum.start() ;

			for _i in 0..100usize
			{
				addr.do_send( AxAdd( 10 ) );
			}

			let res = await!( addr.send( AxShow{} ) ).unwrap();

			assert_eq!( 1005, res );

			actix::System::current().stop();

			Ok(())

		}.boxed().compat());

	});
}


fn actix_send()
{
	actix::System::run( ||
	{
		Arbiter::spawn( async
		{
			let sum  = AxSum(5)    ;
			let addr = sum.start() ;

			for _i in 0..100usize
			{
				await!( addr.send( AxAdd( 10 ) ) ).unwrap();
			}

			let res = await!( addr.send( AxShow{} ) ).unwrap();

			assert_eq!( 1005, res );

			actix::System::current().stop();

			Ok(())

		}.boxed().compat());

	});
}


fn method()
{
	block_on( async
	{
		let mut sum = Accu(5);

		for _i in 0..100usize
		{
			await!( sum.add( Add( 10 ) ) );
		}

		let res = await!( sum.show() );
		assert_eq!( 1005, res );
	})
}


fn inline_method()
{
	block_on( async
	{
		let mut sum = Accu(5);

		for _i in 0..100usize
		{
			await!( sum.add_inline( Add( 10 ) ) );
		}

		let res = await!( sum.show() );
		assert_eq!( 1005, res );
	})
}


// --------------------------------------------------------------------

struct AxSum (u64);
struct AxAdd (u64);
struct AxShow     ;

impl AxMessage for AxAdd  { type Result  = ()              ; }
impl AxMessage for AxShow { type Result  = u64             ; }
impl AxActor   for AxSum  { type Context = AxContext<Self> ; }


impl AxHandler< AxAdd > for AxSum
{
	type Result  = ()
;
	fn handle( &mut self, msg: AxAdd, _ctx: &mut AxContext<Self> )
	{
		self.0 += msg.0;
	}
}


impl AxHandler< AxShow > for AxSum
{
	type Result  = u64
;
	fn handle( &mut self, _msg: AxShow, _ctx: &mut AxContext<Self> ) -> Self::Result
	{
		self.0
	}
}




fn bench_calls( c: &mut Criterion )
{
	c.bench
	(
		"Single Thread Delivery",

		Benchmark::new   ( "Send x100"               , |b| b.iter( || send         () ) )
			.with_function( "Call x100"               , |b| b.iter( || call         () ) )
			.with_function( "async method x100"       , |b| b.iter( || method       () ) )
			.with_function( "async inline method x100", |b| b.iter( || inline_method() ) )
			.with_function( "actix do_send x100"      , |b| b.iter( || actix_dosend () ) )
			.with_function( "actix send x100"         , |b| b.iter( || actix_send   () ) )
	);
}

criterion_group!( benches, bench_calls );
criterion_main! ( benches              );
