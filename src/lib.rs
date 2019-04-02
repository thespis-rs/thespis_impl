#![ allow( unused_imports, dead_code ) ]
#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns ) ]


pub mod single_thread;
pub mod multi_thread;
pub mod runtime;

mod error;

pub use
{
	error::*,
};


mod import
{
	pub use
	{
		failure :: { Error, Fail                                                                        } ,
		std     :: { sync::Arc, pin::Pin, future::Future, marker::PhantomData, cell::RefCell            } ,
		futures :: { prelude::{ Stream, StreamExt, Sink, SinkExt }, channel::{ oneshot, mpsc }          } ,
		futures :: { future::{ FutureExt, TryFutureExt },                                               } ,
		futures :: { task::{ Spawn, SpawnExt, LocalSpawn, LocalSpawnExt }                               } ,
		futures :: { executor::{ LocalPool as LocalPool03, LocalSpawner as LocalSpawner03, ThreadPool as ThreadPool03 }                 } ,

		// crossbeam_channel :: { futures::mpsc as crossbeam                                               } ,
		thespis :: { *                                                                                  } ,
		log     :: { *                                                                                  } ,
		tokio_async_await :: { await as await01                                                         } ,
		tokio_current_thread:: { spawn                                                                  } ,
		once_cell:: { unsync::OnceCell, unsync::Lazy, unsync_lazy                                       } ,
	};
}
