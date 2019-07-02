#![ feature( async_await, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]


use
{
	thespis       :: { *               } ,
	thespis_impl  :: { *               } ,
	async_runtime :: { rt              } ,
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

	}) }
}


impl Handler< Show > for Sum
{
	fn handle( &mut self, _msg: Show ) -> Return<u64> { Box::pin( async move
	{

		self.0

	})}
}



fn main()
{
	let program = async move
	{
		let     sum  = Sum(5);
		let mut addr = Addr::try_from( sum ).expect( "Failed to create address" );

		for _i in 0..10_000_000usize
		{
			addr.call( Add( 10 ) ).await.expect( "Send failed" );
		}

		let res = addr.call( Show{} ).await.expect( "Call failed" );
		assert_eq!( 100_000_005, res );

		dbg!( res );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}
