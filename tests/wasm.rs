#![ cfg( target_arch = "wasm32" ) ]

wasm_bindgen_test_configure!( run_in_browser );

mod common;

use
{
	common            :: { import::*, actors::* } ,
	async_executors   :: { Bindgen              } ,
	wasm_bindgen_test :: { *                    } ,
	wasm_bindgen      :: { *                    } ,

};


#[ wasm_bindgen_test ]
//
async fn stop_when_adresses_dropped_before_start_mb()
{
	// let _ = flexi_logger::Logger::with_str( "trace" ).start();

	let (addr, mb) = Addr::builder().build();

	let addr2 = addr.clone();

	drop( addr  );
	drop( addr2 );

	let mb_handle = Bindgen.spawn_handle( mb.start( Sum(5) ) ).unwrap_throw();

	// the handle will resolve when the mailbox has ended. If not this test will hang.
	//
	mb_handle.await;
}



#[ wasm_bindgen_test ]
//
async fn test_basic_send()
{
	let mut addr = Addr::builder().start( Sum(5), &Bindgen ).unwrap_throw();

	addr.send( Add( 10 ) ).await.unwrap_throw();

	assert_eq!( 15, addr.call( Show ).await.unwrap_throw() );
}



#[ wasm_bindgen_test ]
//
async fn test_basic_call()
{
	let mut addr = Addr::builder().start( Sum(5), &Bindgen ).unwrap_throw();

	addr.call( Add(10) ).await.unwrap_throw();

	assert_eq!( 15, addr.call( Show ).await.unwrap_throw() );
}



#[ wasm_bindgen_test ]
//
async fn send_from_multiple_addrs()
{
	let mut addr  = Addr::builder().start( Sum(5), &Bindgen ).unwrap_throw();
	let mut addr2 = addr.clone();

	addr .send( Add( 10 ) ).await.unwrap_throw();
	addr2.send( Add( 10 ) ).await.unwrap_throw();

	assert_eq!( 25, addr.call( Show{} ).await.unwrap_throw() );
}



#[ wasm_bindgen_test ]
//
async fn call_from_multiple_addrs()
{
	let mut addr  = Addr::builder().start( Sum(5), &Bindgen ).unwrap_throw();
	let mut addr2 = addr.clone();

	addr .call( Add( 10 ) ).await.unwrap_throw();
	addr2.call( Add( 10 ) ).await.unwrap_throw();

	assert_eq!( 25, addr.call ( Show{} ).await.unwrap_throw() );
}
