#![ allow(dead_code) ]

pub use
{
	criterion         :: { Criterion, criterion_group, criterion_main       } ,
	futures           :: { FutureExt, channel::oneshot, task::LocalSpawnExt } ,
	thespis           :: { *                                                } ,
	thespis_impl      :: { *                                                } ,
	std               :: { thread, sync::{ Arc, Mutex }, convert::TryFrom   } ,
	std               :: { marker::PhantomData, rc::Rc                      } ,
	actix             :: { Actor as _, ActorFuture, Arbiter, System         } ,
	futures           :: { executor::block_on                               } ,
	async_executors   :: { *                                                } ,
};

pub type DynError = Box< dyn std::error::Error + Send + Sync + 'static >;

pub const BOUNDED : usize = 16;
pub const MESSAGES: usize = 100_000;

pub const MPSC_BOUNDED: usize = 26;
pub const MPSC_SENDERS: usize = 10;
pub const MPSC_TOTAL  : usize = MESSAGES *10 + 5 + termial( MESSAGES );


pub const fn termial( n: usize ) -> usize
{
	n * (n + 1) / 2
}


#[ derive( Actor, Debug ) ] pub struct Sum
{
	pub total: u64,
	pub inner: Addr<SumIn>,
	pub _nosend: PhantomData<Rc<()>>,
}


#[ derive( Actor, Debug ) ] pub struct SumIn
{
	pub count: u64,
}


pub struct Add ( pub u64 ) ;
pub struct Show            ;

impl Message for Add  { type Return = () ; }
impl Message for Show { type Return = u64; }


impl Handler< Add > for Sum
{
	#[async_fn_local] fn handle_local( &mut self, msg: Add )
	{
		let inner = self.inner.call( Show ).await.expect( "call inner" );

		self.total += msg.0 + inner;
	}

	#[async_fn] fn handle( &mut self, _msg: Add )
	{
		unreachable!( "non send actor" );
	}
}


impl Handler< Show > for Sum
{
	#[async_fn_local] fn handle_local( &mut self, _msg: Show ) -> u64
	{
		self.total
	}

	#[async_fn] fn handle( &mut self, _msg: Show ) -> u64
	{
		unreachable!( "non send actor" );
	}
}


impl Handler< Show > for SumIn
{
	#[async_fn] fn handle( &mut self, _msg: Show ) -> u64
	{
		self.count += 1;
		self.count
	}
}


#[ derive( Debug ) ]
//
pub struct ActixSum
{
	pub total: u64,
	pub inner: actix::Addr<SumIn>,
	pub _nosend: PhantomData<Rc<()>>,
}

impl actix::Actor for ActixSum { type Context = actix::Context<Self>; }
impl actix::Actor for SumIn    { type Context = actix::Context<Self>; }

impl actix::Message for Add  { type Result = () ; }
impl actix::Message for Show { type Result = u64; }


impl actix::Handler< Add > for ActixSum
{
	type Result = actix::ResponseActFuture< Self, <Add as actix::Message>::Result >;

	fn handle( &mut self, msg: Add, _ctx: &mut Self::Context ) -> Self::Result
	{
		let action = self.inner.send( Show );

		let act = actix::fut::wrap_future::<_, Self>(action);

		let update_self = act.map( move |result, actor, _ctx|
		{
			actor.total += msg.0 + result.expect( "Call SumIn" );
		});

		Box::pin( update_self )
	}
}


impl actix::Handler< Show > for ActixSum
{
	type Result = u64;

	fn handle( &mut self, _msg: Show, _ctx: &mut actix::Context<Self> ) -> Self::Result
	{
		self.total
	}
}


impl actix::Handler< Show > for SumIn
{
	type Result = u64;

	fn handle( &mut self, _msg: Show, _ctx: &mut actix::Context<Self> ) -> Self::Result
	{
		self.count += 1;
		self.count
	}
}


pub struct Accu
{
	count: Mutex<u64>    ,
	inner: Mutex<AccuIn> ,
}

impl Accu
{
	#[ inline(never) ]
	//
	async fn add( &self, v: Add )
	{
		let from_in = self.inner.lock().unwrap().show().await;
		let mut count = self.count.lock().unwrap();
		*count += v.0 + from_in;
	}

	#[ inline(never) ]
	//
	async fn show( &self ) -> u64
	{
		*self.count.lock().unwrap()
	}
}


pub struct AccuIn( u64 );

impl AccuIn
{
	#[ inline( never ) ]
	//
	async fn show( &mut self ) -> u64
	{
		self.0 += 1;
		self.0
	}
}
