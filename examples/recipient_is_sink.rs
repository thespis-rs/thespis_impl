use
{
	futures           :: { stream, StreamExt } ,
	thespis           :: { *                 } ,
	thespis_impl      :: { *                 } ,
	futures::executor :: { ThreadPool        } ,
	std               :: { error::Error      } ,
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
async fn main() -> Result< (), Box<dyn Error> >
{
	let     a      = MyActor { count: 0 };
	let     exec   = ThreadPool::new()?;
	let mut addr   = Addr::builder().start( a, &exec )?;
	let     stream = stream::iter( vec![ Count, Count, Count ].into_iter() ).map( Ok );

	stream.forward( &mut addr ).await?;

	let total = addr.call( Count ).await?;

	assert_eq!( 4, total );
	dbg!( total );

	Ok(())
}
