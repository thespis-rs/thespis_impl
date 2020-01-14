use crate::{ import::*, error::* };



/// This type can be used when you need a concrete type as Address<M>. Eg,
/// you can store this as BoxAny and then use down_cast from std::any::Any.
//
pub struct Receiver<M: Message>
{
	rec: Pin<BoxAddress<M, ThesErr>>
}

impl<M: Message> Receiver<M>
{
	/// Create a new Receiver
	//
	pub fn new( rec: BoxAddress<M, ThesErr> ) -> Self
	{
		Self { rec: Pin::from( rec ) }
	}
}



impl<M: Message> Clone for Receiver<M>
{
	fn clone( &self ) -> Self
	{
		Self { rec: Pin::from( self.rec.clone_box() ) }
	}
}



impl<M: Message> fmt::Debug for Receiver<M>
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		write!( f, "Receiver: {:?}", &self.rec )
	}
}


/// Verify whether 2 Receivers will deliver to the same actor
//
impl<M: Message> PartialEq for Receiver<M>
{
	fn eq( &self, other: &Self ) -> bool
	{
		self.rec.actor_id() == other.rec.actor_id()
	}
}

impl<M: Message> Eq for Receiver<M>{}



impl<M: Message> Address<M> for Receiver<M>
{
	fn call( &mut self, msg: M ) -> Return<'_, Result< <M as Message>::Return, <Self as Sink<M>>::Error >>
	{
		Box::pin( async move
		{
			self.rec.call( msg ).await

		})
	}



	fn clone_box( &self ) -> BoxAddress<M, ThesErr>
	{
		self.rec.clone_box()
	}



	fn actor_id( &self ) -> usize
	{
		self.rec.actor_id()
	}
}



impl<M: Message> Sink<M> for Receiver<M>
{
	type Error = ThesErr;

	fn poll_ready( mut self: Pin<&mut Self>, cx: &mut TaskContext<'_> ) -> Poll<Result<(), Self::Error>>
	{
		self.rec.as_mut().poll_ready( cx )
	}


	fn start_send( mut self: Pin<&mut Self>, msg: M ) -> Result<(), Self::Error>
	{
		self.rec.as_mut().start_send( msg )
	}


	fn poll_flush( mut self: Pin<&mut Self>, cx: &mut TaskContext<'_> ) -> Poll<Result<(), Self::Error>>
	{
		self.rec.as_mut().poll_flush( cx )
	}


	/// Will only close when dropped, this method can never return ready
	//
	fn poll_close( mut self: Pin<&mut Self>, cx: &mut TaskContext<'_> ) -> Poll<Result<(), Self::Error>>
	{
		self.rec.as_mut().poll_close( cx )
	}
}
