//! Demonstrates desugaring all convenience methods, showing how you can have full control over
//! all aspects of thespis.
//
use
{
	thespis         :: { Return, Actor, Message, Handler, Address } ,
	thespis_impl    :: { DynError, Mailbox, MailboxEnd            } ,
	async_executors :: { AsyncStd, SpawnHandleExt                 } ,
	std             :: { error::Error                             } ,
	futures         :: { channel::mpsc, FutureExt, SinkExt        } ,
};


#[ derive( Debug, Actor ) ]
//
struct MyActor;


struct Hello( String );

impl Message for Hello
{
	type Return = &'static str;
}


impl Handler< Hello > for MyActor
{
	fn handle( &mut self, _msg: Hello ) -> Return< <Hello as Message>::Return >
	{
		async	{ "world" }.boxed()
	}
}


#[async_std::main]
//
async fn main() -> Result< (), Box<dyn Error> >
{
	// We can use any channel we want, as long as it has Sink+Clone/Stream.
	//
	let (tx, rx)  = mpsc::channel( 5 );

	// We must use a dynamic error for the Sink, so Addr can store it, whichever
	// error type our channel actually uses.
	//
	let tx = Box::new( tx.sink_map_err( |e| Box::new(e) as DynError ) );

	// Manually create a mailbox, with a name for the actor and the receiver of
	// our channel.
	//
	let mb = Mailbox::new( "HelloWorld", Box::new(rx) );

	// The mailbox gives us the address.
	//
	let mut addr = mb.addr( tx );

	// We didn't need our actor state until now, so if you wanted it to have it's
	// own address, that's perfectly feasible to pass in the constructor.
	//
	let actor = MyActor;


	// When the mb starts, it takes ownership of our actor.
	//
	let mb_handle = AsyncStd.spawn_handle( mb.start(actor) )?;

	// call is fallible. The mailbox might be closed (actor panicked), or
	// it might close after accepting the message but before sending a response.
	// See `ThesErr` docs.
	//
	let result = addr.call( Hello( "hello".into() ) ).await?;

	assert_eq!( "world", dbg!(result) );

	// The mailbox stops when all addresses have been dropped. There is also `WeakAddr`
	// for addresses that don't keep the mailbox alive.
	//
	drop( addr );

	// When the mailbox stops, we get our actor state back.
	//
	assert!( matches!( dbg!(mb_handle.await), MailboxEnd::Actor(MyActor) ) );

	Ok(())
}
