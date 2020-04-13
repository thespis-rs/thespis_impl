// Demonstrates how to process messages concurrently if no mutable state is needed.
//
use
{
	thespis           :: { *                                    } ,
	thespis_impl      :: { *                                    } ,
	std               :: { error::Error                         } ,
	futures           :: { task::{ Spawn, SpawnExt }, FutureExt } ,
	async_executors   :: { AsyncStd                             } ,
};


type DynError = Box<dyn Error + Send + Sync>;

#[ derive( Actor ) ]
//
struct MyActor
{
	exec: Box< dyn Spawn + Send >
}

struct Ping;

impl Message for Ping {	type Return = Result<(), DynError >; }


impl Handler<Ping> for MyActor
{
	fn handle( &mut self, _msg: Ping ) -> Return<'_, <Ping as Message>::Return >
	{
		// self had properties wrapped in Arc, we could clone them here to pass them
		// into the future.
		//
		// However we can't actually pass the reference to self in here if we want it
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
		// You can even imagine using spawn_handle and storing the JoinHandles somewhere
		// (eg. FuturesUnordered). Every now and then you could poll the FuturesUnordered
		// so it verifies how many tasks are still running, or return the JoinHandle to the
		// caller By awaiting it in the async block we return, you can return the result of
		// processing to your caller without blocking the current actor while it runs.
		// The sky is the limit.
		//
		let result = self.exec.spawn( processing );

		// If spawning failed, we pass that back to caller.
		// We are now immediately ready to process the next message even while processing is
		// still running.
		//
		async move { result.map_err( |e| Box::new(e) as DynError ) }.boxed()
	}
}


#[async_std::main]
//
async fn main() -> Result< (), DynError >
{
	let mut addr = Addr::builder().start( MyActor{ exec: Box::new(AsyncStd) }, &AsyncStd )?;

	let result = addr.call( Ping ).await?;

	dbg!( result )?;

	Ok(())
}
