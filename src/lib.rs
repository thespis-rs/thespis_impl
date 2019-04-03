#![ allow( unused_imports, dead_code ) ]
#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro ) ]


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
		failure   :: { Error, Fail                                                                     } ,
		std       :: { sync::Arc, pin::Pin, future::Future, marker::PhantomData, cell::RefCell, rc::Rc } ,
		thespis   :: { *                                                                               } ,
		log       :: { *                                                                               } ,
		tokio     :: { await as await01                                                                } ,
		once_cell :: { unsync::OnceCell, unsync::Lazy, unsync_lazy                                     } ,

		futures ::
		{
			prelude :: { Stream, StreamExt, Sink, SinkExt           } ,
			channel :: { oneshot, mpsc                              } ,
			future  :: { FutureExt, TryFutureExt                    } ,
			task    :: { Spawn, SpawnExt, LocalSpawn, LocalSpawnExt } ,

			executor::
			{
				LocalPool    as LocalPool03    ,
				LocalSpawner as LocalSpawner03 ,
				ThreadPool   as ThreadPool03   ,
			},
		},
	};
}
