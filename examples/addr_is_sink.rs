//! Demonstrates that Addresses are Sinks. We can forward a stream into them.
//
use
{
	futures           :: { stream, StreamExt } ,
	thespis           :: { *                 } ,
	thespis_impl      :: { *                 } ,
	std               :: { error::Error      } ,
	async_executors   :: { AsyncStd          } ,
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
	let mut addr   = Addr::builder( "My actor" ).spawn( a, &AsyncStd )?;

	// Create an ad hoc stream.
	//
	let stream = stream::iter( vec![ Count, Count, Count ].into_iter() ).map( Ok );

	// Have the actor process all.
	//
	stream.forward( &mut addr ).await?;

	let total = addr.call( Count ).await?;

	assert_eq!( 4, total );
	dbg!( total );

	Ok(())
}
