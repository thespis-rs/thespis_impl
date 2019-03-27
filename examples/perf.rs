#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

#![ allow( dead_code, unused_imports )]


use
{
	futures       :: { future::{ Future, FutureExt }, task::{ LocalSpawn, SpawnExt }, executor::block_on } ,
	std           :: { pin::Pin                                                                            } ,
	log           :: { *                                                                                   } ,
	thespis       :: { *                                                                                   } ,
	thespis_impl  :: { *                                                                                   } ,
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
	block_on( async
	{
		let     sum                      = Sum(5)      ;
		let mut mb  : ProcLocalMb  <Sum> = sum.start() ;
		let mut addr: ProcLocalAddr<Sum> = mb .addr () ;

		for _i in 0..100usize
		{
			await!( addr.call( Add( 10 ) ) );
		}

		let res = await!( addr.call( Show{} ) );
		assert_eq!( 1005, res );

		dbg!( res );
	})
}
