use
{
	thespis         :: { *                         } ,
	thespis_impl    :: { *                         } ,
	async_executors :: { AsyncStd, SpawnHandleExt  } ,
	std             :: { error::Error              } ,
	futures         :: { channel::mpsc             } ,
};


#[ derive( Actor ) ]
//
struct MyActor;


struct Hello( String );

impl Message for Hello
{
	type Return = String;
}


impl Handler< Hello > for MyActor
{
	#[async_fn]	fn handle( &mut self, _msg: Hello ) -> String
	{
		"world".into()
	}
}


#[async_std::main]
//
async fn main() -> Result< (), Box<dyn Error> >
{
	let (tx, rx)  = mpsc::channel( 5 );
	let mb        = Mailbox::new( Some("HelloWorld".into()), Box::new(rx) );
	let mut addr  = Addr::new( mb.id(), mb.name(), Box::new( tx.sink_map_err( |e| Box::new(e) as SinkError ) ) );
	let actor     = MyActor;

	let handle = AsyncStd.spawn_handle( mb.start( actor ) )?;

	let result = addr.call( Hello( "hello".into() ) ).await?;

	assert_eq!( "world", dbg!(result) );

	drop( addr );
	handle.await;

	Ok(())
}
