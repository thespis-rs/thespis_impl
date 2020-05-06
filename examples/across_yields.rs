// Demonstrates modifying mutable state accross await points. No synchronization needed.

use
{
	log               :: { *            } ,
	thespis           :: { *            } ,
	thespis_impl      :: { *            } ,
	futures::executor :: { ThreadPool   } ,
	std               :: { error::Error } ,
};


#[ derive( Actor ) ]
//
struct MyActor { seed: String }

impl MyActor
{
	async fn bla( x: &mut String ) -> String
	{
		x.extend( "bla".chars() );
		x.clone()
	}
}


struct Ping( String );


impl Message for Ping { type Return = String; }


impl Handler< Ping > for MyActor
{
	#[async_fn] fn handle( &mut self, msg: Ping ) -> String
	{
		trace!( "Ping handler called" );

		self.seed.extend( msg.0.chars() );
		self.seed = Self::bla( &mut self.seed ).await;
		self.seed += " - after yield";
		self.seed.clone()
	}
}


#[async_std::main]
//
async fn main() -> Result< (), Box<dyn Error> >
{
	simple_logger::init()?;

	let     exec  = ThreadPool::new()?;
	let     a     = MyActor{ seed: "seed".into() };
	let mut addr  = Addr::builder().start( a, &exec )?;
	let mut addr2 = addr.clone();

	trace!( "calling addr.call( Ping( 'ping' ) )" );
	let result = addr.call( Ping( "ping".into() ) ).await?;

	trace!( "calling addr.call( Ping( 'pang' ) )" );
	let result2 = addr2.call( Ping( "pang".into() ) ).await?;

	info!( "We got a result: {}", result );
	assert_eq!( "seedpingbla - after yield".to_string(), result );

	info!( "We got a result: {}", result2 );
	assert_eq!( "seedpingbla - after yieldpangbla - after yield".to_string(), result2 );

	Ok(())
}
