//! # Thespis reference implementation
//!
//! ## Cargo Features
//!
//! - tokio: makes the tokio executor available. enabled by default.
//!
#![deny(bare_trait_objects)]


#![ feature
(
	arbitrary_self_types   ,
	async_await            ,
	box_into_pin           ,
	box_patterns           ,
	box_syntax             ,
	core_intrinsics        ,
	decl_macro             ,
	never_type             ,
	nll                    ,
	optin_builtin_traits   ,
	re_rebalance_coherence ,
	specialization         ,
	todo_macro             ,
	trait_alias            ,
	try_trait              ,
	unboxed_closures       ,
)]

    mod address  ;
    mod envelope ;
    mod mailbox  ;
    mod receiver ;

pub mod runtime  ;


pub use
{
	address  :: * ,
	mailbox  :: * ,
	receiver :: * ,
};





// Import module. Avoid * imports here. These are all the foreign names that exist throughout
// the crate. The must all be unique.
// Separate use imports per enabled features.
//
mod import
{
	pub use
	{
		failure   :: { Fail, bail, err_msg, AsFail, Context as FailContext, Backtrace, ResultExt } ,
		thespis   :: { *                                           } ,
		log       :: { *                                           } ,
		once_cell :: { unsync::OnceCell, unsync::Lazy, unsync_lazy } ,


		std ::
		{
			fmt                                                       ,
			cell        :: { RefCell                                } ,
			convert     :: { TryFrom, TryInto                       } ,
			future      :: { Future                                 } ,
			marker      :: { PhantomData                            } ,
			ops         :: { Try, DerefMut                          } ,
			pin         :: { Pin                                    } ,
			rc          :: { Rc                                     } ,
			sync        :: { Arc, atomic::{ AtomicUsize, Ordering } } ,
			collections :: { HashMap                                } ,

		},


		futures ::
		{
			prelude :: { Stream, StreamExt, Sink, SinkExt                                         } ,
			channel :: { oneshot, mpsc                                                            } ,
			future  :: { FutureExt, TryFutureExt                                                  } ,
			task    :: { Spawn, SpawnExt, LocalSpawn, LocalSpawnExt, Context as TaskContext, Poll } ,

			executor::
			{
				LocalPool    as LocalPool03    ,
				LocalSpawner as LocalSpawner03 ,
				ThreadPool   as ThreadPool03   ,
			},
		},
	};


	#[ cfg( feature = "tokio" ) ]
	//
	pub use
	{
		tokio :: { prelude::{ AsyncRead as TokioAsyncR, AsyncWrite as TokioAsyncW }          } ,
		tokio :: { codec::{ Decoder, Encoder, Framed, FramedParts, FramedRead, FramedWrite } } ,
	};


	#[ cfg(test) ]
	//
	pub use
	{
		pretty_assertions::{ assert_eq, assert_ne } ,
	};
}
