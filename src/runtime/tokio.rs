use
{
	crate :: { import::*, *                                      } ,
	tokio :: { await as await01, runtime::current_thread as trt  } ,
};


/// An executor that uses tokio executor under the hood.
///
/// TODO: threadpool impl. Currently puts everything on tokio_current_thread.
//
pub struct TokioRT
{
	spawned: Rc<RefCell<Vec< Pin<Box< dyn Future< Output = () > + 'static >>>>> ,
	running: RefCell<bool>                                                      ,
}



impl Default for TokioRT
{
	fn default() -> Self
	{
		TokioRT
		{
			spawned: Rc::new( RefCell::new( vec![] ) ),
			running: RefCell::new( false )            ,
		}
	}
}



impl Executor for TokioRT
{
	type Error = ThesErr;

	/// Run all spawned futures to completion.
	//
	fn run( &self )
	{
		let spawned = self.spawned.clone();

		let task = async move
		{
			let mut v = spawned.borrow_mut();

			for fut in v.drain(..)
			{
				trt::spawn( Box::pin( async { await!( fut ); Ok(()) } ).compat() );
			}

			Ok(())

		};

		{ *self.running.borrow_mut() = true; }

		trt::run( Box::pin( task ).compat() );
	}


	/// Spawn a future to be run on the LocalPool (current thread)
	//
	fn spawn( &self, fut: Pin<Box< dyn Future< Output = () > + 'static >> ) -> Result<(), Self::Error>
	{
		if *self.running.borrow()
		{
			trt::spawn( Box::pin( async { await!( fut ); Ok(()) } ).compat() );
		}

		else
		{
			self.spawned.borrow_mut().push( fut );
		}

		Ok(())
	}


	/// Spawn a future to be run on a threadpool.
	/// Not implemented!
	//
	fn spawn_pool( &self, _fut: Pin<Box< dyn Future< Output = () > + 'static >> ) -> Result<(), Self::Error>
	{
		todo!()
	}
}
