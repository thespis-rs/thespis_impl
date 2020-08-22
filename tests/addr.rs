// Tested:
//
// - verify actor stops when all adresses are dropped.

mod common;

use
{
	thespis         :: { *                                } ,
	thespis_impl    :: { *,                               } ,
	common          :: { actors::{ Sum, Add }, import::*  } ,
	futures         :: { executor::block_on, future::join } ,
};




// TODO: this test is a bit of non-sense. This has nothing to do with adresses, but
// depends entirely on the channel. We don't even need to construct an adress to
// make this work. Since we are generic over the channel, this could work with some
// channels and not others.
//
// It's good to have at least one test.
//
#[async_std::test]
//
async fn stop_when_adresses_dropped_before_start_mb()
{
	// let _ = flexi_logger::Logger::with_str( "trace" ).start();

	// Create mailbox
	//
	let (tx, rx) = mpsc::unbounded()                          ;
	let name     = Some( "Sum".into() )                       ;
	let mb       = Mailbox::new( name.clone(), Box::new(rx) ) ;
	let id       = mb.id()                                    ;
	let sum      = Sum(5)                                     ;

	let program = async move
	{
		let tx    = Box::new(tx.sink_map_err( |e| Box::new(e) as SinkError )) ;
		let addr  = Addr ::new( id, name, tx )                                ;
		drop( addr )                                                          ;
	};

	join( program, mb.start( sum ) ).await;
}


// TODO: Maybe need a more realistic test. Here we await the mailbox while usually it will be
// spawned, and we use join which has it's own polling strategy.
//
#[test]
//
fn stop_when_adresses_dropped()
{
	// let _ = flexi_logger::Logger::with_str( "trace" ).start();

	// Create mailbox
	//
	let (tx, rx) = mpsc::unbounded()                          ;
	let name     = Some( "Sum".into() )                       ;
	let mb       = Mailbox::new( name.clone(), Box::new(rx) ) ;
	let id       = mb.id()                                    ;
	let sum      = Sum(5)                                     ;

	let dropper = async move
	{
		let      tx    = Box::new(tx.sink_map_err( |e| Box::new(e) as SinkError )) ;
		let  mut addr  = Addr ::new( id, name, tx )                                ;
		let     _addr2 = addr.clone()                                              ;

		addr.send( Add( 10 ) ).await.expect( "Send failed" );
	};

	let mailbox = async move
	{
		mb.start( sum ).await;
	};

	block_on( join( mailbox, dropper ) );
}

