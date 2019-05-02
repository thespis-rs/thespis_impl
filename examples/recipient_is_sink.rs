#![ feature( await_macro, async_await, arbitrary_self_types, box_syntax, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

use
{
	futures       :: { stream, sink::SinkExt } ,
	thespis       :: { *                     } ,
	thespis_impl  :: { *, runtime::rt        } ,
};



#[ derive( Actor ) ] struct MyActor{ count: u8 }

struct Count;
impl Message for Count { type Return = u8; }


impl Handler< Count > for MyActor
{
	fn handle( &mut self, _msg: Count ) -> ReturnNoSend<u8> { Box::pin( async move
	{
		self.count += 1;
		self.count

	})}
}


fn main()
{
	let program = async move
	{
		let a = MyActor { count: 0 };

		let mut addr    = Addr::try_from( a ).expect( "Failed to create address" );
		let mut stream  = stream::iter( vec![ Count, Count, Count ].into_iter() );

		await!( addr.send_all( &mut stream ) ).expect( "drain stream" );

		// This doesn't really work, we don't support it because:
		//
		// - stream needs to be a TryStream
		// - the future will only complete when the sink is closed, but our addresses can only
		//   close when they are dropped.
		//
		// await!( stream.forward( &mut addr ) ).expect( "forward to sink" );

		let total = await!( addr.call( Count ) ).expect( "Call failed" );

		assert_eq!( 4, total );
		dbg!( total );
	};

	rt::spawn( program ).expect( "Spawn program" );
	rt::run();
}
