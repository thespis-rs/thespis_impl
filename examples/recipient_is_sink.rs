use
{
	futures           :: { stream, StreamExt    } ,
	thespis           :: { *                    } ,
	thespis_impl      :: { *                    } ,
	futures::executor :: { block_on, ThreadPool } ,
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
		let     exec   = ThreadPool::new().expect( "create threadpool" );
		let mut addr   = Addr::try_from( a, &exec ).expect( "Failed to create address" );
		let     stream = stream::iter( vec![ Count, Count, Count ].into_iter() ).map( |i| Ok(i) );

		stream.forward( &mut addr ).await.expect( "forward to sink" );

		let total = addr.call( Count ).await.expect( "Call failed" );

		assert_eq!( 4, total );
		dbg!( total );
	};

	block_on( program );
}
