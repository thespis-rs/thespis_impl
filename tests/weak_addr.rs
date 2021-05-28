// Tested:
//
// ✔ send using weak addr
// ✔ verify error on weak after count = zero
// ✔ upgrade
// ✔ refuse upgrade after mailbox close
// ✔ verify correctly closing mailbox when strong addresses are dropped
//   ✔ when address dropped while channel is pending
//   ✔ when address dropped while stuff in queue
//
#![ cfg(not( target_arch = "wasm32" )) ]


mod common;

use
{
	common          :: { import::*, *, actors::* } ,
	async_executors :: { AsyncStd                } ,
};


#[async_std::test]
//
async fn weak_basic_use() -> Result<(), DynError >
{
	let     addr = Addr::builder().start( Sum(5), &AsyncStd )?;
	let mut weak = addr.weak();

	weak.send( Add( 10 ) ).await?;

	assert_eq!( 15, weak.call( Show ).await? );

	Ok(())
}


#[async_std::test]
//
async fn weak_plenty() -> Result<(), DynError >
{
	let     addr  = Addr::builder().start( Sum(5), &AsyncStd )?;
	let mut weak  = addr.weak();
	let     weak2 = addr.weak();
	let    _weak3 = addr.weak();
	let    _weak4 = addr.weak();
	let    _weak5 = addr.weak();
	let    _weak6 = addr.weak();
	let    _weak7 = addr.weak();

	weak.send( Add( 10 ) ).await?;

	// Some more completely useless addresses
	//
	let _strong  = weak2.strong();
	let _strong2 = weak2.strong();
	let _strong3 = weak2.strong();
	let _strong4 = weak2.strong();
	let _strong5 = weak2.strong();


	assert_eq!( 15, weak.call( Show ).await? );

	Ok(())
}


#[async_std::test]
//
async fn weak_refuse() -> Result<(), DynError >
{
	let     addr = Addr::builder().start( Sum(5), &AsyncStd )?;
	let mut weak = addr.weak();

	let addr2 = weak.strong();
	let addr3 = addr2.clone();

	drop( addr  );
	drop( addr2 );
	drop( addr3 );

	let res = weak.send( Add( 10 ) ).await;

	assert!(matches!( res, Err( ThesErr::MailboxClosed{..} ) ));

	Ok(())
}


#[async_std::test]
//
async fn weak_upgrade() -> Result<(), DynError >
{
	let     addr    = Addr::builder().start( Sum(5), &AsyncStd )?;
	let     weak    = addr.weak();
	let mut upgrade = weak.strong()?;

	upgrade.send( Add( 10 ) ).await?;

	assert_eq!( 15, upgrade.call( Show ).await? );

	Ok(())
}


#[async_std::test]
//
async fn weak_upgrade_refuse() -> Result<(), DynError >
{
	let addr = Addr::builder().start( Sum(5), &AsyncStd )?;
	let weak = addr.weak();

	drop(addr);
	let res = weak.strong();

	assert!(matches!( res, Err( ThesErr::MailboxClosed{..} ) ));

	Ok(())
}







#[async_std::test]
//
async fn strong_drop_close_mailbox() -> Result<(), DynError >
{
	let (addr, mb_handle) = Addr::builder().start_handle( Sum(5), &AsyncStd )?;
	let _weak = addr.weak();

	drop(addr);

	// This should not hang, as the mailbox should close when the last strong addr is dropped.
	//
	mb_handle.await;

	Ok(())
}



#[async_std::test]
//
async fn weak_drop_dont_close_mailbox() -> Result<(), DynError >
{
	let mut addr = Addr::builder().start( Sum(5), &AsyncStd )?;
	let     weak = addr.weak();

	drop(weak);
	addr.send( Add( 10 ) ).await?;

	assert_eq!( 15, addr.call( Show ).await? );

	Ok(())
}






#[ derive( Actor ) ] pub struct Blocked( Arc<Mutex<()>> );
#[ derive( Debug ) ] pub struct Hi;

impl Message for Hi { type Return = ();  }

impl Handler<Hi> for Blocked
{
	#[async_fn] fn handle( &mut self, _msg: Hi )
	{
		let _lock = self.0.lock();
	}
}



#[async_std::test]
//
async fn drop_strong_while_mb_pending() -> Result<(), DynError >
{
	let shared  = Arc::new(Mutex::new( () ));
	let shared2 = shared.clone();

	let     (addr, mb) = Addr::builder().bounded( Some(1) ).build();
	let mut weak   = addr.weak();


	// We need to make sure that the mailbox doesn't run on the same thread as this
	// function, otherwise the mutex will deadlock.
	//
	let _handle = std::thread::spawn( move ||
	{
		let exec = TokioCtBuilder::new().build().unwrap();

		exec.block_on( mb.start( Blocked(shared2) ) );
	});


	// First one gets pulled out of the channel and blocks in the handler.
	// Second one fills the one slot in the channel.
	//
	let lock = shared.lock();

	weak.send( Hi ).await?;
	weak.send( Hi ).await?;

	// Drop while messages waiting in the queue.
	//
	drop(addr);
	drop(lock);

	let res = weak.send( Hi ).await;


	assert!(matches!( res, Err( ThesErr::MailboxClosed{..} ) ));

	Ok(())
}
