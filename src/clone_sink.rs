use crate::import::*;

/// Interface for T: Sink + Clone
//
pub trait CloneSink<'a, Item, E>: Sink<Item, Error=E> + Unpin + Send + Sync
{
	/// Clone this sink.
	//
	fn clone_sink( &self ) -> Box< dyn CloneSink<'a, Item, E> + 'a >;
}


impl<'a, T, Item, E> CloneSink<'a, Item, E> for T

	where T: 'a + Sink<Item, Error=E> + Clone + Unpin + Send + Sync + ?Sized

{
	fn clone_sink( &self ) -> Box< dyn CloneSink<'a, Item, E> + 'a >
	{
		Box::new( self.clone() )
	}
}
