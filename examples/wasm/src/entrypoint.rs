#![ feature( async_await ) ]

use wasm_bindgen::prelude::*;


use
{
	async_runtime:: { rt },
	thespis      :: { *  },
	thespis_impl :: { *  },
};


struct Ping(usize);

impl Message for Ping { type Return = usize; }

/// Actor
//
#[ derive( Actor ) ]
//
struct MyActor { count: usize }



/// Handler for `Ping` message
//
impl Handler<Ping> for MyActor
{
	fn handle( &mut self, msg: Ping ) -> Return<<Ping as Message>::Return>
	{
		Box::pin( async move
		{
			self.count += msg.0;
			self.count
		})
	}
}



// Called when the wasm module is instantiated
//
#[ wasm_bindgen( start ) ]
//
pub fn main() -> Result<(), JsValue>
{
	console_log::init_with_level( log::Level::Trace ).expect( "initialize logger" );

	let program = async move
	{
		// start new actor
		//
		let mut addr = Addr::try_from( MyActor { count: 10 } ).expect( "create addres for MyActor" );

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
	};


	rt::spawn( program ).expect( "spawn program" );

	Ok(())
}
