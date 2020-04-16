#![allow(dead_code)]

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

pub type DynError = Box< dyn std::error::Error + Send + Sync >;
