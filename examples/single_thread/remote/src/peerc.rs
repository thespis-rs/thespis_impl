#![ allow( unused_imports, dead_code ) ]
#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, slice_concat_ext ) ]

mod common;
use common::*;

fn main()
{
	rt::init( box TokioRT::default() ).expect( "We only set the executor once" );
	let _handle = flexi_logger::Logger::with_str( "thespis_impl=debug, tokio=info" ).start().unwrap();

	let program = async move
	{
		trace!( "Starting peerC" );

		let peerb = await!( connect_to_tcp( "127.0.0.1:8999" ) );


		// Call the service and receive the response
		//
		let mut service_a = PeerAServices::recipient::<ServiceA>( peerb.clone() );
		let mut service_b = PeerAServices::recipient::<ServiceB>( peerb         );



		let resp = await!( service_a.call( ServiceA{ msg: "hi from peerC".to_string() } ) ).expect( "Call failed" );

		dbg!( resp );



		// Send
		//
		await!( service_b.send( ServiceB{ msg: "SEND from peerC".to_string() } ) ).expect( "Send failed" );



		let resp = await!( service_a.call( ServiceA{ msg: "hi from peerC -- again!!!".to_string() } ) ).expect( "Call failed" );

		dbg!( resp );


		// Send
		//
		await!( service_b.send( ServiceB{ msg: "SEND AGAIN from peerC".to_string() } ) ).expect( "Send failed" );
	};

	rt::spawn( program ).expect( "Spawn program" );

	rt::run();
}
