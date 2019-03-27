#![ allow( unused_imports, dead_code ) ]
#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns ) ]


mod address;
mod envelope;
mod mailbox;

pub use
{
	address   :: * ,
	envelope  :: * ,
	mailbox   :: * ,
};


mod import
{
	pub use
	{
		std     :: { sync::Arc, pin::Pin, future::Future, marker::PhantomData } ,
		futures :: { prelude::{ Stream, StreamExt, Sink, SinkExt }, channel::{ oneshot, mpsc }, future::FutureExt, executor::ThreadPool, task::SpawnExt } ,

		thespis :: { *                                   } ,
		log     :: { *                                   } ,
	};
}
