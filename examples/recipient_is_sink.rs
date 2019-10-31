use
{
	futures           :: { stream, sink::SinkExt } ,
	thespis           :: { *                     } ,
	thespis_impl      :: { *                     } ,
	async_executors   :: { *                     } ,
	futures::executor :: { block_on              } ,
};



#[ derive( Actor ) ] struct MyActor{ count: u8 }

struct Count;
impl Message for Count { type Return = u8; }


impl Handler< Count > for MyActor
{
	fn handle( &mut self, _msg: Count ) -> Return<u8> { Box::pin( async move
	{
		self.count += 1;
		self.count

	})}
}


fn main()
{
	let program = async move
	{
		let     a      = MyActor { count: 0 };
		let mut exec   = ThreadPool::new().expect( "create threadpool" );
		let mut addr   = Addr::try_from( a, &mut exec ).expect( "Failed to create address" );
		let mut stream = stream::iter( vec![ Count, Count, Count ].into_iter() );

		addr.send_all( &mut stream ).await.expect( "drain stream" );

		// This doesn't really work, we don't support it because:
		//
		// - stream needs to be a TryStream
		// - the future will only complete when the sink is closed, but our addresses can only
		//   close when they are dropped.
		//
		// stream.forward( &mut addr ).await.expect( "forward to sink" );

		let total = addr.call( Count ).await.expect( "Call failed" );

		assert_eq!( 4, total );
		dbg!( total );
	};

	block_on( program );
}
