// Tested:
//
// ✔ Manually supervise
// ✔ Use an actor as a supervisor
//
#![ cfg(not( target_arch = "wasm32" )) ]


mod common;

use
{
	common::import::*,
};



#[ derive( Actor ) ] struct Counter;

struct Add(usize);


impl Message for Add  { type Return = usize; }


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
#[ derive( Actor ) ] struct Supervisor { /*self_addr: Addr<Self>,*/ exec: Box< dyn Spawn + Send> }

struct Supervise<A: Actor>
{
	inbox : Option< JoinHandle<MailboxEnd<A>> > ,
	create: Box< dyn FnMut() ->A + Send >            ,
}


// https://github.com/rust-lang/futures-rs/issues/2211
//
impl<A: Actor + Send> std::panic::UnwindSafe for Supervise<A> {}

impl<A: Actor + Send> Message for Supervise<A> { type Return = Option< Addr<A> >; }


impl<A: Actor + Send> Handler< Supervise<A> > for Supervisor
{
	#[async_fn] fn handle( &mut self, mut actor: Supervise<A> ) -> Option< Addr<A> >
	{
		let mut addr = None;

		let mut mb_handle = if actor.inbox.is_none()
		{
			let (addr_new, mb_handle) = Addr::builder().spawn_handle( (actor.create)(), &AsyncStd ).unwrap();

			addr = Some(addr_new);

			mb_handle
		}

		else { actor.inbox.take().unwrap() };


		let supervisor = async move
		{
			while let MailboxEnd::Mailbox(mb) = mb_handle.await
			{
				mb_handle = AsyncStd.spawn_handle( mb.start( (actor.create)() ) ).unwrap();
			}
		};

		self.exec.spawn( supervisor ).unwrap();

		addr
	}
}



// Manually supervise
//
#[async_std::test]
//
async fn supervise() -> Result< (), DynError >
{
	let (mut addr, mut mb_handle) = Addr::builder().spawn_handle( Counter, &AsyncStd )?;


	let supervisor = async move
	{
		while let MailboxEnd::Mailbox(mb) = mb_handle.await
		{
			mb_handle = AsyncStd.spawn_handle( mb.start( Counter ) ).unwrap();
		}
	};


	AsyncStd.spawn( supervisor )?;

	assert!(matches!( addr.call( Add(10) ).await, Err( ThesErr::ActorStoppedBeforeResponse{..} ) ));
	assert!(matches!( addr.send( Add(10) ).await, Ok(()) ));

	assert_eq!( addr.call( Add(11) ).await, Ok(11) );

	Ok(())
}



// Use an actor as supervisor
//
#[async_std::test]
//
async fn supervisor() -> Result< (), DynError >
{
	let mut supervisor = Addr::builder().spawn( Supervisor{ exec: Box::new( AsyncStd ) }, &AsyncStd )?;

	let supervise = Supervise
	{
		create: Box::new( || Counter ),
		inbox: None,
	};

	let mut addr = supervisor.call( supervise ).await?.unwrap();

	assert!(matches!( addr.call( Add(10) ).await, Err( ThesErr::ActorStoppedBeforeResponse{..} ) ));
	assert!(matches!( addr.send( Add(10) ).await, Ok(()) ));

	assert_eq!( addr.call( Add(11) ).await, Ok(11) );

	Ok(())
}
