// Demonstrates how to process messages concurrently if no mutable state is needed.
//
use
{
	thespis           :: { *                                                 } ,
	thespis_impl      :: { *                                                 } ,
	std               :: { error::Error                                      } ,
	futures           :: { FutureExt, task::{ SpawnError }                   } ,
	async_executors   :: { AsyncStd, SpawnHandle, SpawnHandleExt, JoinHandle } ,
	async_nursery     :: { Nursery, NurseErr, Nurse, NurseExt                } ,
};


type DynError = Box<dyn Error + Send + Sync>;

#[ derive( Actor ) ]
//
struct MyActor
{
	nursery: Box< dyn Nurse<usize> + Send >,
	_nursery_handle: JoinHandle<()>,
}

struct Ping;

impl Message for Ping {	type Return = Result<(), NurseErr>; }


impl MyActor
{
	pub fn new( exec: impl SpawnHandle<usize> + SpawnHandle<()> + Clone + Send + 'static ) -> Result<Self, SpawnError>
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

			// Return something if you want.
			//
			5
		};

		// Processing will now run concurrently.
		// You can even imagine using spawn_handle and storing the JoinHandles somewhere
		// (eg. FuturesUnordered). Every now and then you could poll the FuturesUnordered
		// so it verifies how many tasks are still running, or return the JoinHandle to the
		// caller By awaiting it in the async block we return, you can return the result of
		// processing to your caller without blocking the current actor while it runs.
		// The sky is the limit.
		//
		let result = self.nursery.nurse( processing );

		// If spawning failed, we pass that back to caller.
		// We are now immediately ready to process the next message even while processing is
		// still running.
		//
		// We return the result of processing to the caller. If you want this actor to process
		// the outcome of processing, or if you need to guarantee that no more processing tasks
		// are running when this actor get's dropped, you can use a nursery. See: the async_nursery
		// crate.
		//
		async move
		{
			match result
			{
				Ok(handle) => Ok( handle ) ,
				Err(err)   => Err(err)           ,
			}

		}.boxed()
	}
}


#[async_std::main]
//
async fn main() -> Result< (), DynError >
{
	let actor = MyActor::new( Box::new(AsyncStd) )?;
	let mut addr = Addr::builder().start( actor, &AsyncStd )?;

	// Admittedly, this looks a bit weird. Call is fallible, and it returns a result over
	// the SpawnError, since the handler needs to spawn and spawning is fallible.
	//
	let result = addr.call( Ping ).await??;

	dbg!( result );

	Ok(())
}