use
{
	thespis           :: { *            } ,
	thespis_impl      :: { *            } ,
	futures::executor :: { ThreadPool   } ,
	std               :: { error::Error } ,
};


#[ derive( Actor ) ] struct MyActor;
#[ derive( Actor ) ] struct Other  ;

struct Ping( String );


impl Message for Ping
{
	type Return = String;
}


impl Handler< Ping > for MyActor
{
	#[async_fn] fn handle( &mut self, _msg: Ping ) -> String
	{
		"MyActor".into()
	}
}


impl Handler< Ping > for Other
{
	#[async_fn] fn handle( &mut self, _msg: Ping ) -> String
	{
		"Other".into()
	}
}


#[async_std::main]
//
async fn main() -> Result< (), Box<dyn Error> >
{
	let exec  = ThreadPool::new()?;
	let addr  = Addr::builder().start( MyActor, &exec )?;
	let addro = Addr::builder().start( Other  , &exec )?;


	let recs: Vec< BoxAddress<Ping, ThesErr> > = vec![ Box::new( addr ), Box::new( addro ) ];

	for mut actor in recs
	{
		println!( "Pinged: {}", actor.call( Ping( "ping".into() ) ).await? );
	}

	Ok(())
}
