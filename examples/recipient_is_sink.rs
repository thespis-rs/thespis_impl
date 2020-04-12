use
{
	futures           :: { stream, StreamExt } ,
	thespis           :: { *                 } ,
	thespis_impl      :: { *                 } ,
	futures::executor :: { ThreadPool        } ,
	// async_executors   :: { ThreadPool        } ,
};



#[ derive( Actor ) ] struct MyActor{ count: u8 }

struct Count;
impl Message for Count { type Return = u8; }


impl Handler< Count > for MyActor
{
	#[async_fn] fn handle( &mut self, _msg: Count ) -> u8
	{
		self.count += 1;
		self.count
	}
}


#[async_std::main]
//
async fn main()
{
	let     a      = MyActor { count: 0 };
	let     exec   = ThreadPool::new().expect( "create threadpool" );
	let mut addr   = Addr::try_from_actor( a, &exec ).expect( "Failed to create address" );
	let     stream = stream::iter( vec![ Count, Count, Count ].into_iter() ).map( Ok );

	stream.forward( &mut addr ).await.expect( "forward to sink" );

	let total = addr.call( Count ).await.expect( "Call failed" );

	assert_eq!( 4, total );
	dbg!( total );
}
