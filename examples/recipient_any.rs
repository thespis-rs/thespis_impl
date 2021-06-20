//! See how we can have a collection of addresses to differnt actors. In this example
//! we want to store several actors, but we also want to be able to send them different
//! kinds of messages.
//
use
{
	thespis           :: { *                                            } ,
	thespis_impl      :: { *                                            } ,
	async_executors   :: { AsyncStd                                     } ,
	std               :: { error::Error, collections::HashMap, any::Any } ,
};


#[ derive( Actor ) ] struct Actor1;
#[ derive( Actor ) ] struct Actor2;


struct Ping;

impl Message for Ping { type Return = (); }

struct Pong;

impl Message for Pong { type Return = (); }


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
	let addr1: Addr<Actor1> = Addr::builder().spawn( Actor1, &AsyncStd )?;
	let addr2: Addr<Actor2> = Addr::builder().spawn( Actor2, &AsyncStd )?;

	// pub type BoxAddress<M, E> = Box< dyn Address<M, Error=E> + Send + Unpin >;
	// Type can be elided here. It's just there for illustrative purposes.
	//
	let mut col: HashMap<&'static str, Box<dyn Any>> = HashMap::new();

	// Obviously if we have a known number of types, we could use an enum. If
	// not, we can use a dynamic type.
	//
	// With Box<dyn Any> we can store both concrete types of Addr, or also
	// Box< dyn Address<M, ThesErr> > If we want to store addresses without
	// having to know the explicit actor type, we can store them based on the
	// message type we want to send them.
	//
	col.insert( "Address<Ping>", Box::new( addr1.clone_box() ) );
	col.insert( "Actor1"       , Box::new( addr1 )             );
	col.insert( "Actor2"       , Box::new( addr2 )             );

	for (k, mut v) in col
	{
		// As we erase all type information with Any, we now need to store
		// the type we need to downcast to some other way.
		//
		match k
		{
			"Actor1" =>
			{
				let addr: &mut Addr<Actor1> = v.downcast_mut().unwrap();

				addr.send( Ping ).await?;
			}

			"Actor2" =>
			{
				let addr: &mut Addr<Actor2> = v.downcast_mut().unwrap();

				addr.send( Ping ).await?;
			}


			"Address<Ping>" =>
			{
				let addr: &mut BoxAddress<Ping, ThesErr> = v.downcast_mut().unwrap();

				addr.send( Ping ).await?;
			}

			_ => unreachable!(),
		}
	}

	Ok(())
}
