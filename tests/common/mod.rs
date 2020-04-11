pub mod actors;


pub mod import
{
	pub use
	{
		futures       :: { future::{ FutureExt }, SinkExt, channel::mpsc } ,
		thespis       :: { * } ,
		log           :: { * } ,
		std           :: { marker::PhantomData } ,
	};
}
