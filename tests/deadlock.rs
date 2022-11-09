//! Test for the gateway type deadlock. Verify that by using a priority channel
//! we can avoid it.
//!
//! When an actor sits on the intersection between incoming requests and outgoing responses,
//! like when it's managing a network connection, a deadlock can arise.
//!
//! If processing cannot keep up with incoming requests, the inbox of the gate will fill up.
//! This will provide back pressure. If the solution to this is to send some response out
//! to free up space, an issue arises as the gate mailbox wont take any outgoing message as
//! it's already full and the gate actor itself is not taking anything out of it's mailbox
//! as it's currently blocked on an incoming message.
//!
//! In the most simple scenario we can work around this by using a priority channel for the
//! gate mailbox. This way even if the incoming side is fully backed up, a worker actor can
//! still deliver the outgoing response to the gate. It will now take its next message out of
//! it's mailbox and the free slot will propagate back up to the gate. It will now get it's
//! next message and as the mailbox is a priority channel, the outgoing side will get handled
//! first. Even a buffer of one message on the outgoing side is enough to prevent a deadlock.
//!
//! The downside is that this only works in this most simple use case. As soon as you have
//! more than one source of new messages (either more connections sending requests, or new
//! messages being created internally), you cannot guarantee the free slot propagates to the
//! gate that received the response.
//!
//! My conclusion is that in a more complex system using channels for backpressure is not
//! an option. Using a semaphore to keep track of how many requests are currently being
//! processed by the system is a better solution.
//!
//! This test is an implementation of the priority channel solution. Note that this does
//! not work with futures channels as they don't guarantee to wake up a sender when
//! a message is read from a reader.
//
#![allow(clippy::suspicious_else_formatting)]
use
{
	tracing           :: { *                                                     } ,
	thespis           :: { *                                                     } ,
	thespis_impl      :: { *                                                     } ,
	async_chanx       :: { tokio::mpsc                                           } ,
	async_executors   :: { AsyncStd, SpawnHandleExt                              } ,
	std               :: { error::Error, future::Future, pin::Pin                } ,
	futures           :: { stream::{ select_with_strategy, PollNext }, FutureExt } ,
	async_progress    :: { Progress                                              } ,
};




const BOUNDED: usize = 1;

pub type DynResult<T> = Result< T, Box<dyn Error> >;


#[ derive( Actor ) ] struct Gate
{
	worker: WeakAddr<Worker>   ,
}


#[ derive( Actor ) ] struct Worker
{
	gate  : Addr<Gate>         ,
	steps : Progress<GateStep> ,
	send_out: Option<Pin<Box< dyn Future<Output=()> + Send >>>,
}


struct Request (usize);
struct Response(usize);

impl Message for Request  { type Return = (); }
impl Message for Response { type Return = (); }



impl Handler< Request > for Gate
{
	#[async_fn] fn handle( &mut self, req: Request )
	{
		info!( "Gate: New request: {}.", req.0 );

		if req.0 == 2
		{
			let mut f = self.worker.send( Request(req.0) );

			let f2 = futures::future::poll_fn( |cx: &mut std::task::Context|
			{
				let p = Pin::new( &mut f ).poll(cx);
				debug!( "result of trying to send: {:?}", &p );
				p
			});

			f2.await.expect("send");
		}

		else
		{
			self.worker.send( Request(req.0) ).await.expect( "send" );
		}

		info!( "Gate: Successfully sent: {}.", req.0 );
	}
}



impl Handler< Response > for Gate
{
	#[async_fn] fn handle( &mut self, resp: Response )
	{
		info!( "Gate: Sending reponse to request: {}.", resp.0 );
	}
}



impl Handler< Request > for Worker
{
	#[async_fn] fn handle( &mut self, work: Request )
	{
		info!( "Worker: Grinding on request: {}.", work.0 );

		if work.0 == 1
		{
			self.steps.set_state( GateStep::BackedUp ).await;
		}


		info!( "Worker: Waiting for SendOut." );
		if let Some(f) = self.send_out.take() { f.await; }
		info!( "Worker: Green light from SendOut." );

		self.gate.send( Response(work.0) ).await.expect( "send" );
		info!( "Worker: Response sent." );

	}
}


// Steps we need to take.
//
// - send BOUNDED *2 + 2 messages. This should fill the both mailboxes and
//   both actors should have a message ready to go out.
// - verify that that the mailbox in for gate gives back pressure
// - verify that we can send out with the mailbox out of gate.
// - verify that we take the next message out of our mailbox in worker, this now
//   allows gate to forward the message it was waiting with.
// - now gate can take the next message from it's mailbox which should prioritize the outgoing one.

// Some steps in our flow.
//
#[ derive( Debug, Clone, PartialEq, Eq )]
//
enum GateStep
{
   Fill,
   BackedUp,
   SendOut,
}


#[ async_std::test ]
//
async fn deadlock() -> DynResult<()>
{
	let _ = tracing_subscriber::fmt::Subscriber::builder()

		.with_max_level(tracing::Level::TRACE)
		.with_env_filter( "trace" )
		.json()
		.try_init()
	;

	let steps     = Progress::new( GateStep::Fill );
	let backed_up = steps.once( GateStep::BackedUp );
	let send_out  = steps.once( GateStep::SendOut ).map(|_|());

	let ( low_tx,  low_rx) = mpsc::channel( BOUNDED );
	let (high_tx, high_rx) = mpsc::channel( BOUNDED );

	let strategy = |_: &mut ()| PollNext::Left;
	let gate_rx  = Box::new( select_with_strategy( high_rx, low_rx, strategy ) );

	let gate_low_tx  = low_tx .sink_map_err( |e| Box::new(e) as DynError );
	let gate_high_tx = high_tx.sink_map_err( |e| Box::new(e) as DynError );

	let     gate_mb        = Mailbox::new( "gate", gate_rx );
	let mut gate_low_addr  = gate_mb.addr( Box::new( gate_low_tx  ) );
	let     gate_high_addr = gate_mb.addr( Box::new( gate_high_tx ) );

	let (worker_addr, worker_mb) = Addr::builder().bounded( Some(BOUNDED) ).build();

	let gate   = Gate   { worker: worker_addr.weak()                                                      };
	let worker = Worker { gate  : gate_high_addr, steps: steps.clone(), send_out: send_out.boxed().into() };

	let   gate_handle = AsyncStd.spawn_handle(   gate_mb.start(   gate ) )?;
	let worker_handle = AsyncStd.spawn_handle( worker_mb.start( worker ) )?;

	// Fill both queues and one message for each actor.
	//
	let fill = BOUNDED*2 + 2;

	for i in 1..=fill
	{
		info!( "main: prepping request: {i}." );
		gate_low_addr.send( Request(i) ).await?;
		info!( "main: sent request: {i}." );
	}

	backed_up.await;
	steps.set_state( GateStep::SendOut ).await;

	info!( "main: sent SendOut." );

	for i in (fill+1)..10
	{
		gate_low_addr.send( Request(i) ).await?;
	}

	// This synchronizes the last one so we don't drop the addresses to early.
	//
	gate_low_addr.call( Request(10) ).await?;


	drop(gate_low_addr);
	drop(worker_addr);

	worker_handle.await;
	gate_handle.await;

	Ok(())
}
