//! Demonstrates modifying mutable state accross await points. No synchronization needed.
//
use
{
	tracing           :: { *            } ,
	thespis           :: { *            } ,
	thespis_impl      :: { *            } ,
	std               :: { error::Error } ,
	async_executors   :: { AsyncStd     } ,
};


#[ derive( Actor ) ]
//
struct MyActor { seed: String }

impl MyActor
{
	async fn bla( x: &mut String )
	{
		x.push_str( " - bla" )
	}
}


struct Ping( String );


impl Message for Ping { type Return = String; }


impl Handler< Ping > for MyActor
{
	#[async_fn] fn handle( &mut self, msg: Ping ) -> String
	{
		trace!( "Ping handler called" );

		// mutate in handler.
		//
		self.seed.push_str( &msg.0 );

		// async operation can mutate our state.
		//
		Self::bla( &mut self.seed ).await;

		// mutate again after yielding.
		//
		self.seed += " - after yield";

		self.seed.clone()
	}
}


#[async_std::main]
//
async fn main() -> Result< (), Box<dyn Error> >
{
	tracing_subscriber::fmt::Subscriber::builder()

	   .with_max_level(tracing::Level::TRACE)
	   .init()
	;


	let     a     = MyActor{ seed: "seed - ".into() };
	let mut addr  = Addr::builder().start( a, &AsyncStd )?;
	let mut addr2 = addr.clone();

	trace!( "calling addr.call( Ping('ping') )" );
	let result = addr.call( Ping( "ping".into() ) ).await?;

	trace!( "calling addr.call( Ping('pang') )" );
	let result2 = addr2.call( Ping( " - pang".into() ) ).await?;

	info!( "We got a result: {}", result );
	assert_eq!( "seed - ping - bla - after yield".to_string(), result );

	info!( "We got a result: {}", result2 );
	assert_eq!( "seed - ping - bla - after yield - pang - bla - after yield".to_string(), result2 );

	Ok(())
}
