//! # Thespis reference implementation
//!
//! ## Cargo Features
//!
//! - tokio: makes the tokio executor available. enabled by default.
//!
#![deny(bare_trait_objects)]

#![ feature( core_intrinsics ) ]

mod address  ;
mod envelope ;
mod inbox    ;
mod receiver ;


pub use
{
	address  :: *  ,
	inbox    :: *  ,
	receiver :: *  ,
};





// Import module. Avoid * imports here. These are all the foreign names that exist throughout
// the crate. The must all be unique.
// Separate use imports per enabled features.
//
mod import
{
	pub(crate) use
	{
		failure       :: { ResultExt } ,
		thespis       :: { *         } ,
		log           :: { *         } ,
		async_runtime :: { rt        } ,

		std ::
		{
			fmt ,
			pin         :: { Pin                               } ,
			sync        :: { atomic::{ AtomicUsize, Ordering } } ,
		},


		futures ::
		{
			stream  :: { StreamExt                    } ,
			sink    :: { Sink, SinkExt                } ,
			channel :: { oneshot, mpsc                } ,
			future  :: { FutureExt                    } ,
			task    :: { Context as TaskContext, Poll } ,
		},
	};



	#[ cfg(test) ]
	//
	pub use
	{
		pretty_assertions::{ assert_eq, assert_ne } ,
	};
}
