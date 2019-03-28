#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

#![ allow( dead_code, unused_imports )]


use
{
	futures       :: { future::{ Future, FutureExt, TryFutureExt }, task::{ LocalSpawn, SpawnExt }, executor::LocalPool } ,
	std           :: { pin::Pin                                                                                         } ,
	log           :: { *                                                                                                } ,
	thespis       :: { *                                                                                                } ,
	thespis_impl  :: { single_thread::*                                                                                 } ,
	tokio_current_thread:: { spawn, block_on_all                                                                        } ,
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



fn main()
{
	let mut pool = LocalPool::new();
	let mut exec = pool.spawner();

	let program = async move
	{
		let     sum                      = Sum(5)      ;
		let mut mb  : Inbox  <Sum> = sum.start( &mut exec ) ;
		let mut addr: Addr<Sum> = mb .addr () ;

		for _i in 0..10_000_000usize
		{
			await!( addr.call( Add( 10 ) ) );
		}

		let res = await!( addr.call( Show{} ) );
		assert_eq!( 100_000_005, res );

		dbg!( res );

	};

	pool.run_until( program );
}
