//! See how we can have a collection of addresses to differnt actors. In this example
//! we want to store several actors that all accept the same message type.
//
use
{
	thespis           :: { *            } ,
	thespis_impl      :: { *            } ,
	async_executors   :: { AsyncStd     } ,
	std               :: { error::Error } ,
};


#[ derive( Actor ) ] struct Actor1;
#[ derive( Actor ) ] struct Actor2;


struct Ping;

impl Message for Ping { type Return = (); }


impl Handler< Ping > for Actor1
{
	#[async_fn]	fn handle( &mut self, _msg: Ping ) {}
}


impl Handler< Ping > for Actor2
{
	#[async_fn]	fn handle( &mut self, _msg: Ping ) {}
}


#[async_std::main]
//
async fn main() -> Result< (), Box<dyn Error> >
{
	// As you can see, the addresses are generic over the actor type.
	// So we can't just store them in a collection.
	//
	let addr1: Addr<Actor1> = Addr::builder().start( Actor1, &AsyncStd )?;
	let addr2: Addr<Actor2> = Addr::builder().start( Actor2, &AsyncStd )?;

	// pub type BoxAddress<M, E> = Box< dyn Address<M, Error=E> + Send + Unpin >;
	// Type can be elided here. It's just there for illustrative purposes.
	//
	// Obviously if we have a known number of types, we could use an enum. If
	// not, we can use a dynamic type.
	//
	let col: Vec<BoxAddress<Ping, ThesErr>> = vec!
	[
		addr1.clone_box() ,
		addr2.clone_box() ,
	];


	for mut addr in col
	{
		addr.send( Ping ).await?;
	}

	Ok(())
}
