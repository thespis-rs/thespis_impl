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

mod addr       ;
mod clone_sink ;
mod envelope   ;
mod error      ;
mod inbox      ;
mod receiver   ;


pub use
{
	addr       :: * ,
	clone_sink :: * ,
	error      :: * ,
	inbox      :: * ,
	receiver   :: * ,
};


/// Type of boxed channel sender for Addr.
//
pub type ChanSender<A> = Box< dyn CloneSink< 'static, thespis::BoxEnvelope<A>, futures::channel::mpsc::SendError > >;

/// Type of boxed channel sender for Addr.
//
pub type ChanReceiver<A> = Box< dyn futures::Stream<Item=thespis::BoxEnvelope<A>> + Send + Sync + Unpin >;



// Import module. Avoid * imports here. These are all the foreign names that exist throughout
// the crate. The must all be unique.
// Separate use imports per enabled features.
//
mod import
{
	pub(crate) use
	{
		thiserror     :: { Error } ,
		thespis       :: { *     } ,
		log           :: { *     } ,

		std ::
		{
			fmt         :: { self                                   } ,
			pin         :: { Pin                                    } ,
			sync        :: { Arc, atomic::{ AtomicUsize, Ordering } } ,
		},


		futures ::
		{
			stream  :: { StreamExt                                       } ,
			sink    :: { Sink, SinkExt                                   } ,
			channel :: { oneshot, mpsc                                   } ,
			future  :: { FutureExt                                       } ,
			task    :: { Context as TaskContext, Poll, Spawn, SpawnExt, LocalSpawn, LocalSpawnExt } ,
		},
	};
}
