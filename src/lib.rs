// See: https://github.com/rust-lang/rust/issues/44732#issuecomment-488766871
//!
#![ cfg_attr( feature = "external_doc", feature(external_doc)         ) ]
#![ cfg_attr( feature = "external_doc", doc(include = "../README.md") ) ]
//
#![ doc    ( html_root_url = "https://docs.rs/thespis_impl" ) ]
#![ deny   ( missing_docs                                   ) ]
#![ forbid ( unsafe_code                                    ) ]
#![ allow  ( clippy::suspicious_else_formatting             ) ]

#![ warn
(
	missing_debug_implementations ,
	missing_docs                  ,
	nonstandard_style             ,
	rust_2018_idioms              ,
	trivial_casts                 ,
	trivial_numeric_casts         ,
	unused_extern_crates          ,
	unused_qualifications         ,
	single_use_lifetimes          ,
	unreachable_pub               ,
	variant_size_differences      ,
)]

mod actor_builder ;
mod addr          ;
mod clone_sink    ;
mod envelope      ;
mod error         ;
mod inbox         ;
mod receiver      ;


pub use
{
	actor_builder :: * ,
	addr          :: * ,
	clone_sink    :: * ,
	error         :: * ,
	inbox         :: * ,
	receiver      :: * ,
};


/// Shorthand for a `Send` boxed envelope.
//
pub type BoxEnvelope<A> = Box< dyn envelope::Envelope<A>  + Send >;


/// A boxed error type for the sink
//
pub type SinkError = Box< dyn std::error::Error + Send + 'static >;

/// Type of boxed channel sender for Addr.
//
pub type ChanSender<A> = Box< dyn CloneSink< 'static, BoxEnvelope<A>, SinkError> >;

/// Type of boxed channel receiver for Inbox.
//
pub type ChanReceiver<A> = Box< dyn futures::Stream<Item=BoxEnvelope<A>> + Send + Unpin >;



// Import module. Avoid * imports here. These are all the foreign names that exist throughout
// the crate. The must all be unique.
// Separate use imports per enabled features.
//
mod import
{
	pub(crate) use
	{
		thiserror       :: { Error   } ,
		thespis         :: { *       } ,
		log             :: { *       } ,
		async_executors :: { SpawnHandle, SpawnHandleExt, LocalSpawnHandle, LocalSpawnHandleExt, JoinHandle } ,

		std ::
		{
			fmt                                                  ,
			pin    :: { Pin                                    } ,
			sync   :: { Arc, atomic::{ AtomicUsize, Ordering } } ,
			panic  :: { AssertUnwindSafe                       } ,
		},


		futures ::
		{
			stream  :: { StreamExt                                       } ,
			sink    :: { Sink, SinkExt                                   } ,
			channel :: { oneshot                                         } ,
			future  :: { FutureExt                                       } ,
			task    :: { Context as TaskContext, Poll, Spawn, SpawnExt, LocalSpawn, LocalSpawnExt } ,
		},
	};
}
