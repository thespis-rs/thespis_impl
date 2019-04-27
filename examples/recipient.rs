#![ feature( await_macro, async_await, arbitrary_self_types, box_syntax, specialization, nll, never_type, unboxed_closures, trait_alias ) ]

use
{
	futures       :: { future::{ FutureExt }  } ,
	thespis       :: { *                      } ,
	thespis_impl  :: { *, runtime::rt         } ,
};


#[ derive( Actor ) ] struct MyActor;
#[ derive( Actor ) ] struct Other  ;

struct Ping( String );


impl Message for Ping
{
	type Return = String;
}


impl Handler< Ping > for MyActor
{
	fn handle( &mut self, _msg: Ping ) -> Return<String> { async move
	{
		"MyActor".into()

	}.boxed() }
}


impl Handler< Ping > for Other
{
	fn handle( &mut self, _msg: Ping ) -> Return<String> { async move
	{
		"Other".into()

	}.boxed() }
}



fn main()
{
	let program = async move
	{
		let a = MyActor;
		let b = Other;

		let addr  = Addr::try_from( a ).expect( "Failed to create address" );
		let addro = Addr::try_from( b ).expect( "Failed to create address" );


		let recs: Vec<Box< Recipient<Ping> >> = vec![ box addr, box addro ];

		// or like this, but it clones internally. Note that the compiler is capable here of detecting
		// that we want a Recipient to the message type Ping.
		//
		// let recs = vec![ addr.recipient(), addro.recipient() ];
		//
		// In a more complex situation, it might be necessary to annotate the type.

		for mut actor in recs
		{
			println!( "Pinged: {}", await!( actor.call( Ping( "ping".into() ) ) ).expect( "Call failed" ) );
		}
	};

	rt::spawn( program ).expect( "Spawn program" );

	rt::run();
}
