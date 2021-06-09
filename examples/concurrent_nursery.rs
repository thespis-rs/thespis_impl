// Demonstrates how to process messages concurrently if no mutable state is needed.
// This time we use a nursery to make sure that none of the spawned subtasks outlive
// the lifetime of our actor.
//
use
{
	thespis           :: { *                                                 } ,
	thespis_impl      :: { *                                                 } ,
	futures           :: { FutureExt, task::{ SpawnError }                   } ,
	async_executors   :: { AsyncStd, SpawnHandle, SpawnHandleExt, JoinHandle } ,
	async_nursery     :: { Nursery, NurseErr, Nurse, NurseExt                } ,
};


#[ derive( Actor ) ]
//
struct MyActor
{
	// We store the nursery_handle on our actor. That way the lifetime of all
	// tasks inside the nursery is tied to the lifetime of our actor. If we drop
	// the actor, all subtasks that are still running will be dropped.
	//
	// Alternatively we could have a Handler for a Stop message, which would await
	// the nursery_handle to wait for all subtasks to finish before dropping this actor.
	// That last one can even be combined with a timeout to limit how long we wait for
	// subtasks to finish naturally before dropping them.
	//
	nursery: Box< dyn Nurse<()> + Send >,
	_nursery_handle: JoinHandle<()>,
}


struct Ping;

impl Message for Ping {	type Return = Result<(), NurseErr>; }


impl MyActor
{
	pub fn new( exec: impl SpawnHandle<()> + Clone + Send + 'static ) -> Result<Self, SpawnError>
	{
		let (nursery, output) = Nursery::new( Box::new( exec.clone() ) );

		let _nursery_handle = exec.spawn_handle( output )?;
		let nursery         = Box::new( nursery );

		Ok( Self
		{
			 nursery        ,
			_nursery_handle ,
		})
	}
}


impl Handler<Ping> for MyActor
{
	// For this usecase we don't use the `async_fn` macro since we don't want our entire
	// handler to be in the returned future. We want to set some thing up first.
	//
	fn handle( &mut self, _msg: Ping ) -> Return<'_, <Ping as Message>::Return >
	{
		// If self had properties wrapped in Arc, we could clone them here to pass them
		// into the future.
		//
		// However we can't actually pass the reference to self in if we want it
		// to run concurrently.
		//
		// In theory you can have something in an Arc<Mutex>> or atomic variables, but
		// you really must be careful. As these run concurrently, values will change in the
		// middle of processing which is a footgun.
		//
		// If we would need to update self with the result of an async operation, we can
		// store our own address on self, clone that to pass it into the spawned task
		// and send the result to ourselves.
		//
		let processing = async move
		{
			// do something useful.
		};

		// Processing will now run concurrently.
		//
		let result = self.nursery.nurse( processing );

		// If spawning failed, we pass that back to caller.
		// We are now immediately ready to process the next message even while processing is
		// still running.
		//
		// We return the result of processing to the caller.
		//
		async move { result }.boxed()
	}
}


#[async_std::main]
//
async fn main() -> Result< (), DynError >
{
	let     actor = MyActor::new( Box::new(AsyncStd) )?;
	let mut addr  = Addr::builder().start( actor, &AsyncStd )?;

	// Admittedly, this looks a bit weird. Call is fallible, and it returns a result over
	// the NurseErr, since the handler needs to spawn and spawning is fallible.
	//
	addr.call( Ping ).await??;

	Ok(())
}
