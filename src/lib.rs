#![ cfg_attr( nightly, feature(doc_cfg) ) ]
#![ doc = include_str!("../README.md") ]

#![ doc    ( html_root_url = "https://docs.rs/thespis_impl" ) ]
#![ deny   ( missing_docs                                   ) ]
#![ forbid ( unsafe_code                                    ) ]
#![ allow  ( clippy::suspicious_else_formatting             ) ]

#![ warn
(
	anonymous_parameters          ,
	missing_copy_implementations  ,
	missing_debug_implementations ,
	missing_docs                  ,
	nonstandard_style             ,
	rust_2018_idioms              ,
	single_use_lifetimes          ,
	trivial_casts                 ,
	trivial_numeric_casts         ,
	unreachable_pub               ,
	unused_extern_crates          ,
	unused_qualifications         ,
	variant_size_differences      ,
)]

mod actor_builder ;
mod actor_info    ;
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
	actor_info    :: * ,
	addr          :: * ,
	error         :: * ,
	mailbox       :: * ,
 	weak_addr     :: * ,

	// Addr::send requires SinkExt, so let's re-export that.
	//
	futures::{ SinkExt },
};

pub(crate) use
{
	strong_count ::* ,
	rx_strong    ::* ,
};

use futures::Sink;


/// Shorthand for a `Send` boxed envelope.
//
pub type BoxEnvelope<A> = Box< dyn envelope::Envelope<A> + Send >;

/// A boxed error type for the sink
//
pub type DynError = Box< dyn std::error::Error + Send + Sync >;

/// Type of boxed channel sender for Addr.
/// Can be created conveniently with [`CloneSinkExt::dyned`], but that is rarely
/// needed as you can use the [`ActorBuilder`](ActorBuilder::channel) to override default channels.
//
pub type ChanSender<A> = Box< dyn CloneSink<'static, BoxEnvelope<A>, DynError> >;

/// Type of boxed channel receiver for Mailbox.
//
pub type ChanReceiver<A> = Box< dyn futures::Stream<Item=BoxEnvelope<A>> + Send + Unpin >;


/// Interface for `T: Sink + Clone + Unpin + Send`.
/// This is object safe, so you can clone on a boxed trait. In _thespis_impl_ it is used
/// for the channel sender that goes in the [actor address](Addr).
//
pub trait CloneSink<'a, Item, E>: Sink<Item, Error=E> + Unpin + Send + 'a
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


/// Helper trait to smoothen API for converting a `T: `[`CloneSink`] into [`ChanSender`],
/// which is `Box< dyn CloneSink<'static, BoxEnvelope<A>, DynError> >`.
///
/// Blanket implemented.
//
pub trait CloneSinkExt<A: thespis::Actor, E>
{
	/// Convert a `T: CloneSink` into `ChanSender`, which is
	/// `Box< dyn CloneSink<'static, BoxEnvelope<A>, DynError> >`.
	//
	fn dyned( self ) -> ChanSender<A>

		where Self: CloneSink<'static, BoxEnvelope<A>, E> + Clone + Sized,
		      E: std::error::Error + Sync + Send + 'static
	{
		let closure = |e| -> DynError { Box::new(e) };

		Box::new( self.sink_map_err( closure ) )
	}
}

impl<T, A: thespis::Actor, E> CloneSinkExt<A, E> for T

		where T: Sink<BoxEnvelope<A>, Error=E> + Clone + Unpin + Send + 'static,
		      E: std::error::Error + Sync + Send + 'static
{}


/// Turn into a boxed error
//
pub fn dyn_err<'a, T, Item, E>( sink: T ) -> impl CloneSink<'a, Item, DynError>

	where T: 'a + Sink<Item, Error=E> + Clone + Unpin + Send + ?Sized,
	      E: std::error::Error + Sync + Send + 'static
{
	sink.sink_map_err( |e| -> DynError { Box::new(e) } )
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
			error   :: { Error                                         } ,
			convert :: { TryFrom                                       } ,
			pin     :: { Pin                                           } ,
			sync    :: { Arc, Mutex, atomic::{ AtomicUsize, Ordering } } ,
			panic   :: { AssertUnwindSafe                              } ,
			task    :: { Context as TaskContext, Poll                  } ,
		},


		futures ::
		{
			stream  :: { Stream, StreamExt                                      } ,
			sink    :: { Sink, SinkExt                                          } ,
			future  :: { FutureExt                                              } ,
			task    :: { Spawn, SpawnExt, LocalSpawn, LocalSpawnExt, SpawnError } ,
		},

		oneshot::{ channel as oneshot, Sender as OneSender } ,
	};
}
