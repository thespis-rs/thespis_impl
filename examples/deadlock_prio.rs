//! One of the prominent problems with the actor model are deadlocks.
//! The one we deal with here occurs when we use bounded channels for
//! back pressure. However one actor sits at the gate, processing both
//! incoming and outgoing messages.
//!
//! When the incoming outpace the capacity for processing, the system
//! fills up and when the gate's mailbox is full, outgoing messages
//! cannot get out, leading to a deadlock.
//!
//! The problem is described at length in this blogpost:
//! https://elizarov.medium.com/deadlocks-in-non-hierarchical-csp-e5910d137cc
//!
//! The solution we develop here is rather simple. As thespis uses interfaces
//! like Sink+Stream for communication between addresses and mailboxes, we
//! can simply combine two channels with futures::stream::select_bias so that
//! one of them takes priority. This way we give outgoing messages priority.
//!
use
{
	tracing           :: { *                                 } ,
	thespis           :: { *                                 } ,
	thespis_impl      :: { *                                 } ,
	async_executors   :: { AsyncStd                          } ,
	std               :: { error::Error, time::Duration      } ,
	futures_timer     :: { Delay                             } ,
	futures           :: { stream::{ select_bias, PollNext } } ,
};

static BOUNDED: usize = 5;

pub type DynResult<T> = Result< T, Box<dyn Error> >;


#[ derive( Actor ) ] struct Gate   { worker: WeakAddr<Worker> }
#[ derive( Actor ) ] struct Worker { gate  : Addr<Gate>       }


struct Request(usize);
struct Response(usize);

impl Message for Request  { type Return = (); }
impl Message for Response { type Return = (); }



impl Handler< Request > for Gate
{
	#[async_fn]	fn handle( &mut self, req: Request )
	{
		info!( "Gate: New request: {}.", req.0 );

		self.worker.send( Request(req.0) ).await.expect( "send" );
	}
}



impl Handler< Response > for Gate
{
	#[async_fn]	fn handle( &mut self, resp: Response )
	{
		info!( "Gate: Sending reponse to request: {}.", resp.0 );
	}
}



impl Handler< Request > for Worker
{
	#[async_fn]	fn handle( &mut self, work: Request )
	{
		info!( "Worker: Grinding on request: {}.", work.0 );

		// Processing takes a little bit of time.
		//
		Delay::new( Duration::from_millis(50) ).await;

		self.gate.send( Response(work.0) ).await.expect( "send" );
	}
}



#[ async_std::main ]
//
async fn main() -> DynResult<()>
{
	let _ = tracing_subscriber::fmt::Subscriber::builder()

	   .with_max_level(tracing::Level::TRACE)
	   .with_env_filter( "info,thespis_impl=debug" )
	   // .json()
	   .try_init()
	;

	// The naive implementation deadlocks.
	//
	_naive().await?;

	// The solution.
	//
	fancy().await?;

	Ok(())
}


async fn _naive() -> DynResult<()>
{
	let (mut   gate_addr,   gate_mb) = Addr::builder().bounded( Some(BOUNDED) ).build();
	let (    worker_addr, worker_mb) = Addr::builder().bounded( Some(BOUNDED) ).build();

	let gate   = Gate    { worker: worker_addr.weak() };
	let worker = Worker  { gate  : gate_addr.clone()  };

	let   gate_handle =   gate_mb.start_handle(   gate, &AsyncStd )?;
	let worker_handle = worker_mb.start_handle( worker, &AsyncStd )?;

	for i in 1..100
	{
		gate_addr.send( Request(i) ).await?;
	}

	// This synchronizes the last one so we don't drop the addresses to early.
	//
	gate_addr.call( Request(100) ).await?;

	drop(worker_addr);
	drop(gate_addr);

	worker_handle.await;
	gate_handle.await;

	Ok(())
}


async fn fancy() -> DynResult<()>
{
	// We will create 2 separate channels to the same mailbox.
	// One for high priority (outbound) and one for low priority
	// (inbound).
	//
	// Both will be bounded, but delivering outbound work is always
	// prioritized over taking more inbound work. This will keep the
	// system from congesting.
	//
	// Another solution is to use an unbounded channel for the outbound.
	//
	let ( low_tx,  low_rx) = futures::channel::mpsc::channel( BOUNDED );
	let (high_tx, high_rx) = futures::channel::mpsc::channel( BOUNDED );

	let strategy = |_: &mut ()| PollNext::Left;
	let gate_rx = Box::new( select_bias( high_rx, low_rx, strategy ) );

	let gate_mb = Mailbox::new( Some("gate"), gate_rx );

	let gate_low_tx  = low_tx .sink_map_err( |e| Box::new(e) as SinkError );
	let gate_high_tx = high_tx.sink_map_err( |e| Box::new(e) as SinkError );

	let mut gate_low_addr  = gate_mb.addr( Box::new( gate_low_tx  ) );
	let     gate_high_addr = gate_mb.addr( Box::new( gate_high_tx ) );

	let (worker_addr, worker_mb) = Addr::builder().bounded( Some(BOUNDED) ).build();

	let gate   = Gate   { worker: worker_addr.weak() };
	let worker = Worker { gate  : gate_high_addr     };

	let   gate_handle =   gate_mb.start_handle(   gate, &AsyncStd )?;
	let worker_handle = worker_mb.start_handle( worker, &AsyncStd )?;

	for i in 1..100
	{
		gate_low_addr.send( Request(i) ).await?;
	}

	// This synchronizes the last one so we don't drop the addresses to early.
	//
	gate_low_addr.call( Request(100) ).await?;


	drop(gate_low_addr);
	drop(worker_addr);

	worker_handle.await;
	gate_handle.await;

	Ok(())
}
