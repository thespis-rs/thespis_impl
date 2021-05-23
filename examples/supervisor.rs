//! Demonstrates one way of supervising actors with thespis.
//! The key feature is that the mailbox future will use catch_unwind on your actor and that
//! it will return the mailbox to you if it panics. This way you can instantiate another actor
//! and spawn it again.
//!
use
{
	thespis           :: { *                    } ,
	thespis_impl      :: { *                    } ,
	tracing           :: { *                    } ,
	futures::task     :: { Spawn, SpawnExt      } ,
	std               :: { error::Error         } ,
	async_executors   :: { AsyncStd, JoinHandle } ,
};


#[ derive( Actor ) ] struct Counter;

struct Add(usize);


impl Message for Add  { type Return = usize; }


// This is a silly actor, if you send it an even number, it panics.
// Otherwise it returns your number to you.
//
impl Handler< Add > for Counter
{
	#[async_fn] fn handle( &mut self, msg: Add ) -> usize
	{
		if msg.0 % 2 == 0 { panic!(); }

		msg.0
	}
}


// Actor that can supervise other actors. It will start the actor for the first time
// if it's not already started.
//
#[ derive( Actor ) ]
//
struct Supervisor
{
	// We will need to spawn the mailbox again if the actor panics, so we need an executor.
	// If you want to make this more hierarchical, you can use a nursery from the async_nursery
	// crate to tie all these subtasks to the lifetime of the supervisor and prevent them from
	// getting orphaned.
	//
	exec: Box<dyn Spawn + Send>
}


// The message we will be sending.
//
struct Supervise<A: Actor>
{
	mailbox : Option< JoinHandle<MailboxEnd<A>> > ,

	// A closure that knows how to instantiate the supervised actor.
	//
	create: Box< dyn FnMut() ->A + Send > ,
}

// https://github.com/rust-lang/futures-rs/issues/2211
//
impl<A: Actor + Send> std::panic::UnwindSafe for Supervise<A> {}

impl<A: Actor + Send> Message for Supervise<A>
{
	type Return = Option< Addr<A> >;
}


// Note how the actor type is a generic parameter, so this supervisor works for
// actors of any type.
//
impl<A: Actor + Send> Handler< Supervise<A> > for Supervisor
{
	#[async_fn] fn handle( &mut self, mut actor: Supervise<A> ) -> Option< Addr<A> >
	{
		let mut addr = None;

		let mut mb_handle = if actor.mailbox.is_none()
		{
			let (addr_new, mb_handle) = Addr::builder().start_handle( (actor.create)(), &AsyncStd ).unwrap();

			addr = Some(addr_new);

			mb_handle
		}

		else { actor.mailbox.take().unwrap() };


		let supervisor = async move
		{
			// This is where the magic happens. Every time the handle resolves, we spawn again
			// and replace it with a new handle.
			//
			// When this returns MailboxEnd::Actor, it means the actor has stopped naturally and we don't respawn it.
			//
			while let MailboxEnd::Mailbox(mb) = mb_handle.await
			{
				mb_handle = mb.start_handle( (actor.create)(), &AsyncStd ).unwrap();
			}
		};

		self.exec.spawn( supervisor ).unwrap();

		addr
	}
}


#[async_std::main]
//
async fn main() -> Result< (), Box<dyn Error> >
{
	tracing_subscriber::fmt::Subscriber::builder()

	   .with_max_level( Level::DEBUG )
	   .init()
	;

	let mut supervisor = Addr::builder().start( Supervisor{ exec: Box::new( AsyncStd ) }, &AsyncStd )?;


	// Here we use a closure to create new actors, but if you don't need to capture
	// anything from the environment you might as well just implement `Default` for
	// your actor type.
	//
	let create = Box::new( ||
	{
		debug!( "Creating a new Counter" );
		Counter
	});

	let supervise = Supervise
	{
		create,
		mailbox: None,
	};

	let mut addr = supervisor.call( supervise ).await?.unwrap();

	// Both of these will make the actor panic:
	//
	assert!(matches!( addr.call( Add(10) ).await, Err( ThesErr::ActorStoppedBeforeResponse{..} ) ));
	assert!(matches!( addr.send( Add(10) ).await, Ok(()) ));

	// Yet, our actor is still responding.
	//
	assert_eq!( addr.call( Add(11) ).await, Ok(11) );

	Ok(())
}
