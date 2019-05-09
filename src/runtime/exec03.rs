use crate :: { import::*, * };


/// An executor that uses futures 0.3 executor under the hood.
///
/// TODO: threadpool impl. Currently puts everything on LocalPool.
//
pub struct Exec03
{
	local  : RefCell<LocalPool03>    ,
	spawner: RefCell<LocalSpawner03> ,
	pool   : ThreadPool03            ,
}



impl Default for Exec03
{
	fn default() -> Self
	{
		let local   = LocalPool03 ::new();
		let spawner = local.spawner();

		Exec03
		{
			local  : RefCell::new( local ),
			pool   : ThreadPool03::new().expect( "Create futures::ThreadPool with default configurtion" ),
			spawner: RefCell::new( spawner ),
		}
	}
}



impl Executor for Exec03
{
	type Error = ThesErr;

	/// Run all spawned futures to completion.
	//
	fn run( &self )
	{
		self.local.borrow_mut().run();
	}


	/// Spawn a future to be run on the LocalPool (current thread)
	//
	fn spawn( &self, fut: Pin<Box< dyn Future< Output = () > + 'static >> ) -> Result<(), Self::Error>
	{
		self.spawner.borrow_mut().spawn_local( fut )

			.map_err( |_| ThesErrKind::Spawn{ context: "Executor".into() }.into() )
	}


	/// Spawn a future to be run on a threadpool.
	/// Not implemented!
	//
	fn spawn_pool( &self, _fut: Pin<Box< dyn Future< Output = () > + 'static >> ) -> Result<(), Self::Error>
	{
		todo!()
	}
}
