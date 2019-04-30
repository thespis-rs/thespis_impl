//! # Thespis reference implementation
//!
//! ## Cargo Features
//!
//! - tokio: makes the tokio executor available. enabled by default.
//!
//!
//! ## Actual Features:
//!
//! - Single threaded impl that allows messages that aren't `Send`, and that doesn't pay thread sync overhead
//! - Multi threaded impl for sending messages to actors in different threads
//! - Runtime for convenience (don't have to pass executor around, use static methods `spawn` and `run`)
//! - 2 Executor impls: futures 0.3 executor and tokio executor (TODO: threadpools, currently spawn everything on current thread)
//!
//!
//!
//!
#![ allow( unused_imports, dead_code ) ]

#![ feature
(
	arbitrary_self_types   ,
	async_await            ,
	await_macro            ,
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

    mod address;
    mod envelope;
    mod mailbox;
    mod error;
pub mod runtime;

pub mod remote;


pub use
{
	error::*,
	address::*,
	mailbox::*,
};

// needed for macro
//
pub mod external_deps
{
	pub use once_cell;
	pub use futures;
	pub use thespis;
	pub use serde_cbor;
	pub use serde;
	pub use failure;
}

// Import module. Avoid * imports here. These are all the foreign names that exist throughout
// the crate. The must all be unique.
// Separate use imports per enabled features.
//
mod import
{
	pub use
	{
		failure   :: { Error, Fail, bail, err_msg, AsFail          } ,
		thespis   :: { *                                           } ,
		log       :: { *                                           } ,
		once_cell :: { unsync::OnceCell, unsync::Lazy, unsync_lazy } ,


		std ::
		{
			fmt                             ,
			cell        :: { RefCell          } ,
			convert     :: { TryFrom, TryInto } ,
			future      :: { Future           } ,
			marker      :: { PhantomData      } ,
			ops         :: { Try, DerefMut    } ,
			pin         :: { Pin              } ,
			rc          :: { Rc               } ,
			sync        :: { Arc              } ,
			collections :: { HashMap          } ,
		},


		futures ::
		{
			prelude :: { Stream, StreamExt, Sink, SinkExt           } ,
			channel :: { oneshot, mpsc                              } ,
			future  :: { FutureExt, TryFutureExt                    } ,
			task    :: { Spawn, SpawnExt, LocalSpawn, LocalSpawnExt, Context, Poll } ,

			executor::
			{
				LocalPool    as LocalPool03    ,
				LocalSpawner as LocalSpawner03 ,
				ThreadPool   as ThreadPool03   ,
			},
		},
	};


	#[ cfg( feature = "remote" ) ]
	//
	pub use
	{
		byteorder   :: { LittleEndian, ReadBytesExt, WriteBytesExt           } ,
		bytes       :: { Bytes, BytesMut, Buf, BufMut, IntoBuf               } ,
		num_traits  :: { FromPrimitive, ToPrimitive                          } ,
		num_derive  :: { FromPrimitive, ToPrimitive                          } ,
		rand        :: { Rng                                                 } ,
		std         :: { hash::{ BuildHasher, Hasher }, io::Cursor, any::Any } ,
		twox_hash   :: { RandomXxHashBuilder, XxHash                         } ,
		futures     :: { future::RemoteHandle                                } ,
		pharos      :: { Pharos, Observable                                  } ,

	};


	#[ cfg( feature = "tokio" ) ]
	//
	pub use
	{
		tokio :: { await as await01, prelude::{ AsyncRead as TokioAsyncR, AsyncWrite as TokioAsyncW } } ,
		tokio :: { codec::{ Decoder, Encoder, Framed, FramedParts, FramedRead, FramedWrite } },
	};


	#[ cfg(test) ]
	//
	pub use
	{
		pretty_assertions::{ assert_eq, assert_ne } ,
	};
}
