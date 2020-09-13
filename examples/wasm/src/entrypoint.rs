use wasm_bindgen::prelude::*;

use
{
	thespis         :: { *                           } ,
	thespis_impl    :: { *                           } ,
	async_executors :: { *                           } ,
	std             :: { marker::PhantomData, rc::Rc } ,
};


struct Ping(usize);

impl Message for Ping { type Return = usize; }

// Just to demonstrate we can use !Send actors, since WASM is single threaded anyway.
//
#[ derive( Actor ) ]
//
struct MyActor { count: usize, phantom: PhantomData< Rc<()> > }


/// Handler for `Ping` message
//
impl Handler<Ping> for MyActor
{
	#[async_fn_local] fn handle_local( &mut self, msg: Ping ) -> <Ping as Message>::Return
	{
		self.count += msg.0;
		self.count
	}

	// This is necessary for !Send actors, as handle is a required method.
	//
	#[async_fn] fn handle( &mut self, _: Ping ) -> <Ping as Message>::Return
	{
		unreachable!( "Cannot spawn !Send actor on a threadpool" );
	}
}



// Called when the wasm module is instantiated
//
#[ wasm_bindgen( start ) ]
//
pub async fn main()
{
	console_log::init_with_level( log::Level::Trace ).expect( "initialize logger" );

	// start new actor
	//
	let     a    = MyActor { count: 10, phantom: PhantomData };
	let mut exec = Bindgen{};
	let mut addr = Addr::try_from_actor_local( a, &mut exec ).expect( "create addres for MyActor" );

	// send message and get future for result
	//
	let res = addr.call( Ping(5) ).await.expect( "Send Ping" );

	let window   = web_sys::window  ().expect( "no global `window` exists"        );
	let document = window  .document().expect( "should have a document on window" );
	let body     = document.body    ().expect( "document should have a body"      );

	// Manufacture the element we're gonna append
	//
	let val = document.create_element( "div" ).expect( "Failed to create div" );

	val.set_inner_html( &format!( "The pong value is: {}", res ) );

	body.append_child( &val ).expect( "Coundn't append child" );
}
