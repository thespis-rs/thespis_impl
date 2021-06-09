#![ cfg_attr( nightly, cfg_attr( nightly, doc = include_str!("../README.md") )) ]
#![ doc = "" ] // empty doc line to handle missing doc warning when the feature is missing.
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
mod addr_inner    ;
mod envelope      ;
mod error         ;
mod mailbox       ;
mod rx_strong     ;
mod strong_count  ;
mod weak_addr     ;


pub use
{
	actor_builder :: * ,
	addr          :: * ,
	error         :: * ,
	mailbox       :: * ,
	rx_strong     :: * ,
 	weak_addr     :: * ,

	// Addr::send requires SinkExt, so let's re-export that.
	//
	futures::{ SinkExt },
};

pub(crate) use
{
	strong_count::* ,
};

use futures::Sink;


/// Shorthand for a `Send` boxed envelope.
//
pub type BoxEnvelope<A> = Box< dyn envelope::Envelope<A>  + Send >;

/// A boxed error type for the sink
//
pub type SinkError = Box< dyn std::error::Error + Send + 'static >;

/// Type of boxed channel sender for Addr.
//
pub type ChanSender<A> = Box< dyn CloneSink< 'static, BoxEnvelope<A>, SinkError> >;

/// Type of boxed channel receiver for Mailbox.
//
pub type ChanReceiver<A> = Box< dyn futures::Stream<Item=BoxEnvelope<A>> + Send + Unpin >;


/// Interface for T: Sink + Clone
//
pub trait CloneSink<'a, Item, E>: Sink<Item, Error=E> + Unpin + Send
{
	/// Clone this sink.
	//
	fn clone_sink( &self ) -> Box< dyn CloneSink<'a, Item, E> + 'a >;
}


impl<'a, T, Item, E> CloneSink<'a, Item, E> for T

	where T: 'a + Sink<Item, Error=E> + Clone + Unpin + Send + ?Sized

{
	fn clone_sink( &self ) -> Box< dyn CloneSink<'a, Item, E> + 'a >
	{
		Box::new( self.clone() )
	}
}



// Import module. Avoid * imports here. These are all the foreign names that exist throughout
// the crate. The must all be unique.
// Separate use imports per enabled features.
//
mod import
{
	pub(crate) use
	{
		thespis         :: { * } ,
		tracing         :: { trace, debug, error, error_span, Span } ,
		tracing_futures :: { Instrument } ,
		async_executors :: { SpawnHandle, SpawnHandleExt, LocalSpawnHandle, LocalSpawnHandleExt, JoinHandle } ,

		std ::
		{
			fmt                                                          ,
			convert :: { TryFrom                                       } ,
			pin     :: { Pin                                           } ,
			sync    :: { Arc, Mutex, atomic::{ AtomicUsize, Ordering } } ,
			panic   :: { AssertUnwindSafe                              } ,
			task    :: { Context as TaskContext, Poll                  } ,
		},


		futures ::
		{
			stream  :: { Stream, StreamExt                          } ,
			sink    :: { Sink, SinkExt                              } ,
			channel :: { oneshot                                    } ,
			future  :: { FutureExt                                  } ,
			task    :: { Spawn, SpawnExt, LocalSpawn, LocalSpawnExt } ,
		},
	};
}
