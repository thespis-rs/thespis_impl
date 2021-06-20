#![allow(dead_code)]

pub mod actors;


pub mod import
{
	pub use
	{
		futures :: { future::{ FutureExt }, stream, SinkExt, StreamExt, task::{ Spawn, SpawnExt }, channel::* } ,
		thespis :: { * } ,
		thespis_impl :: { * } ,
		tracing :: { trace, error_span } ,
		tracing_futures::Instrument,
		std     :: { marker::PhantomData, error::Error, sync::{ Arc, Mutex, atomic::Ordering::SeqCst }, num::NonZeroUsize } ,
		async_executors :: { * } ,
	};
}


pub fn init_tracing()
{
	let _ = tracing_subscriber::fmt::Subscriber::builder()

	   .with_max_level(tracing::Level::TRACE)
	   .try_init()
	;
}
