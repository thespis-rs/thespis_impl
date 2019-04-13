#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

// #![ allow( dead_code, unused_imports )]


use
{
	futures       :: { future::{ FutureExt }          } ,
	thespis       :: { *                              } ,
	thespis_impl  :: { single_thread::*, runtime::rt  } ,
};


#[ derive( Actor ) ] struct Sum( u64 );

struct Add (u64);
struct Show     ;

impl Message for Add  { type Result = () ; }
impl Message for Show { type Result = u64; }


impl Handler< Add > for Sum
{
	fn handle( &mut self, msg: Add ) -> TupleResponse { async move
	{

		self.0 += msg.0;

	}.boxed() }
}


impl Handler< Show > for Sum
{
	fn handle( &mut self, _msg: Show ) -> Response<u64> { async move
	{

		self.0

	}.boxed() }
}



fn main()
{
	let program = async move
	{
		let     sum              = Sum(5)      ;

		let mb  : Inbox<Sum> = Inbox::new();
		let send             = mb.sender ();

		mb.start( sum ).expect( "Failed to start mailbox" );

		let mut addr  = Addr::new( send.clone() );


		for _i in 0..10_000_000usize
		{
			await!( addr.call( Add( 10 ) ) ).expect( "Send failed" );
		}

		let res = await!( addr.call( Show{} ) ).expect( "Call failed" );
		assert_eq!( 100_000_005, res );

		dbg!( res );
	};

	rt::spawn( program ).expect( "Spawn program" );

	rt::run();
}
