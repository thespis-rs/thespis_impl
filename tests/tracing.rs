//! Does the span we put for tracing in the mailbox get replaced if we spawn another actor from within the handler?
//! Does the span persist if we spawn other non-actor tasks? Should it?
//!

mod common;

use
{
	thespis         :: { *            } ,
	thespis_impl    :: { *,           } ,
	tracing         :: { *            } ,
	async_executors :: { AsyncStd, SpawnHandleExt, LocalPool, LocalSpawnHandleExt } ,
	futures :: { executor::LocalSpawner     } ,
	std             :: { error::Error } ,
};



#[ derive( Actor ) ] pub struct Creator;
#[ derive( Actor ) ] pub struct Created;
#[ derive( Actor ) ] pub struct Create;

impl Message for Create { type Return = ();  }



impl Handler<Create> for Creator
{
	#[async_fn] fn handle( &mut self, _: Create )
	{
		trace!( "creating new actor" );

		async { trace!( "i am awaited within the handler on Creator" ); }.await;

		let mut created = Addr::builder().start( Created, &AsyncStd ).expect( "create new actor" );

		created.call( Create ).await.expect( "call created" );
	}
}



impl Handler<Create> for Created
{
	#[async_fn] fn handle( &mut self, _: Create )
	{
		trace!( "in created handler" );

		AsyncStd.spawn_handle( async {

			trace!( "a spawned subtask" );

		}).expect( "spawn subtask" ).await;
	}
}



// The purpose of this test is to verify that the tracing span with actor id and name is
// replaced when one actor creates a second one, rather than having both ids.
//
// It requires looking at the log output in order to see it.
//
#[async_std::test]
//
async fn tracing_nested() -> Result<(), Box<dyn Error> >
{
	tracing_subscriber::fmt::Subscriber::builder()

	   .with_max_level(tracing::Level::TRACE)
	   .init()
	;

	let mut addr = Addr::builder().start( Creator, &AsyncStd )?;

	addr.call( Create ).await?;

	Ok(())
}




// Single threaded version, to see if the tracing spans are persistant if the code runs on the same thread.

#[ derive( Actor ) ] pub struct CreatorLocal { exec: LocalSpawner }
#[ derive( Actor ) ] pub struct CreatedLocal { exec: LocalSpawner }




impl Handler<Create> for CreatorLocal
{
	#[async_fn] fn handle( &mut self, _: Create )
	{
		unreachable!();
	}


	#[async_fn_nosend] fn handle_local( &mut self, _: Create )
	{
		trace!( "creating new actor" );

		async { trace!( "i am awaited within the handler on CreatorLocal" ); }.await;


		let mut created = Addr::builder().start_local( CreatedLocal{ exec: self.exec.clone() }, &self.exec ).expect( "create new actor" );

		created.call( Create ).await.expect( "call created" );
	}
}



impl Handler<Create> for CreatedLocal
{
	#[async_fn] fn handle( &mut self, _: Create )
	{
		unreachable!();
	}

	#[async_fn_nosend] fn handle_local( &mut self, _: Create )
	{
		trace!( "in created handler" );

		self.exec.spawn_handle_local( async {

			trace!( "a spawned subtask" );

		}).expect( "spawn subtask" ).await;
	}
}



// The purpose of this test is to verify that the tracing span with actor id and name is
// replaced when one actor creates a second one, rather than having both ids.
//
// It requires looking at the log output in order to see it.
//
#[ test ]
//
fn tracing_nested_local()
{
	tracing_subscriber::fmt::Subscriber::builder()

	   .with_max_level(tracing::Level::TRACE)
	   .init()
	;

	let mut pool = LocalPool::new();
	let exec = pool.spawner();

	let task = async
	{
		let mut addr = Addr::builder().start_local( Creator, &exec ).expect( "start Creator" );

		addr.call( Create ).await.expect( "call Creator" );
	};

	pool.run_until( task );
}
