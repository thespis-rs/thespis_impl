#![ allow( unused_imports, dead_code ) ]
#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns ) ]


pub mod single_thread;


mod import
{
	pub use
	{
		std     :: { sync::Arc, pin::Pin, future::Future, marker::PhantomData                           } ,
		futures :: { prelude::{ Stream, StreamExt, Sink, SinkExt }, channel::{ oneshot, mpsc }          } ,
		futures :: { future::{ FutureExt, TryFutureExt }, executor::LocalSpawner                        } ,
		futures :: { task::{ Spawn, SpawnExt }                                                          } ,

		// crossbeam_channel :: { futures::mpsc as crossbeam                                               } ,
		thespis :: { *                                                                                  } ,
		log     :: { *                                                                                  } ,
		tokio_async_await :: { await as await01                                                         } ,
		tokio_current_thread:: { spawn                                                                  } ,
	};
}
