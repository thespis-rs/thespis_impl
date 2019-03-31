#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

#![ allow( dead_code, unused_imports )]


use
{
	futures       :: { future::{ Future, FutureExt }, task::{ LocalSpawn, Spawn, SpawnExt, LocalSpawnExt }, executor::LocalPool } ,
	thespis       :: { * } ,
	log           :: { * } ,
	thespis_impl  :: { single_thread::* } ,
	std           :: { pin::Pin } ,
};


#[ derive( Actor ) ]
//
struct Sum( u64 );

#[ derive( Debug ) ] struct Add( u64 );
#[ derive( Debug ) ] struct Show;

impl Message for Add  { type Result = ();  }
impl Message for Show { type Result = u64; }



impl Handler< Add > for Sum
{
	fn handle( &mut self, msg: Add ) -> Response<Add> { async move
	{
		trace!( "called sum with: {:?}", msg );

		self.0 += msg.0;

	}.boxed() }
}



impl Handler< Show > for Sum
{
	fn handle( &mut self, _msg: Show ) -> Response<Show> { async move
	{
		trace!( "called sum with: Show" );

		self.0

	}.boxed() }
}



impl Drop for Sum
{
	fn drop( &mut self )
	{
		trace!( "dropping Sum")
	}
}



async fn sum_send( exec: &mut impl LocalSpawn ) -> u64
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new(             );
	let mut addr             = Addr ::new( mb.sender() );

	// This is ugly right now. It will be more ergonomic in the future.
	//
	let move_mb = async move { await!( mb.start( sum ) ); };
	exec.spawn_local( move_mb ).expect( "Spawning mailbox failed" );


	await!( addr.send( Add( 10 ) ) );

	let res = await!( addr.call( Show{} ) );

	trace!( "res is: {}", res );

	res
}



async fn sum_call( exec: &mut impl LocalSpawn ) -> u64
{
	let sum = Sum(5);

	// Create mailbox
	//
	let     mb  : Inbox<Sum> = Inbox::new(             );
	let mut addr             = Addr ::new( mb.sender() );

	// This is ugly right now. It will be more ergonomic in the future.
	//
	let move_mb = async move { await!( mb.start( sum ) ); };
	exec.spawn_local( move_mb ).expect( "Spawning mailbox failed" );


	await!( addr.call( Add( 10 ) ) );

	let res = await!( addr.call( Show{} ) );

	trace!( "res is: {}", res );

	res
}



#[test]
//
fn test_basic_send()
{
	let mut pool = LocalPool::new();
	let mut exec = pool.spawner();

	let program = async move
	{
		let _ = simple_logger::init();

		let result = await!( sum_send( &mut exec ) );

		trace!( "result is: {}", result );
		assert_eq!( 15, result );
	};

	pool.run_until( program );
}



#[test]
//
fn test_basic_call()
{
	let mut pool = LocalPool::new();
	let mut exec = pool.spawner();

	let program = async move
	{
		let _ = simple_logger::init();

		let result = await!( sum_call( &mut exec ) );

		trace!( "result is: {}", result );
		assert_eq!( 15, result );
	};

	pool.run_until( program );
}
