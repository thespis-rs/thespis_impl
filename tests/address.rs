#![ feature( optin_builtin_traits ) ]

// Tested:
//
// - verify actor stops when all adresses are dropped.

mod common;

use
{
	thespis         :: { *                                } ,
	thespis_impl    :: { *,                               } ,
	common          :: { actors::{ Sum }                  } ,
	futures         :: { executor::block_on, future::join } ,
};




#[test]
//
fn stop_when_adresses_dropped_before_start_mb()
{
	// let _ = flexi_logger::Logger::with_str( "trace" ).start();

	// Create mailbox
	//
	let mb    : Inbox<Sum> = Inbox::new() ;
	let sender             = mb.sender()  ;
	let sum                = Sum(5)       ;

	let program = async move
	{
		let  addr  = Addr ::new( sender ) ;
		let _addr2 = addr.clone()         ;
	};

	block_on( join( program, mb.start_fut( sum ) ) );
}


#[test]
//
fn stop_when_adresses_dropped()
{
	// let _ = flexi_logger::Logger::with_str( "trace" ).start();

	// Create mailbox
	//
	let mb    : Inbox<Sum> = Inbox::new() ;
	let sender             = mb.sender()  ;
	let sum                = Sum(5)       ;

	let dropper = async move
	{
		let  addr  = Addr ::new( sender ) ;
		let _addr2 = addr.clone()         ;
	};

	let mailbox = async move
	{
		mb.start_fut( sum ).await;
	};

	block_on( join( mailbox, dropper ) );
}

