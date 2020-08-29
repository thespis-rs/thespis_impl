#![allow(dead_code)]

pub mod actors;


pub mod import
{
	pub use
	{
		futures :: { future::{ FutureExt }, stream, SinkExt, StreamExt, task::{ Spawn, SpawnExt }, channel::* } ,
		thespis :: { * } ,
		thespis_impl :: { * } ,
		tracing :: { * } ,
		std     :: { marker::PhantomData, error::Error } ,
		async_executors :: { * } ,
	};
}

pub type DynError = Box< dyn std::error::Error + Send + Sync >;
