use crate :: { import::*, * };


pub struct Exec03
{
	local  : RefCell<LocalPool03>,
	spawner: RefCell<LocalSpawner03> ,
	pool   : ThreadPool03   ,
}



impl Default for Exec03
{
	fn default() -> Self
	{
		let local   = LocalPool03 ::new();
		let spawner = local.spawner();

		Exec03
		{
			local: RefCell::new( local ),
			pool : ThreadPool03::new().expect( "Create futures::ThreadPool with default configurtion" ),
			spawner: RefCell::new( spawner ),
		}
	}
}



impl Executor for Exec03
{
	fn run( &self )
	{
		self.local.borrow_mut().run();
	}


	fn spawn( &self, fut: Pin<Box< dyn Future< Output = () > + 'static >> ) -> ThesRes<()>
	{
		self.spawner.borrow_mut().spawn_local( fut ).unwrap();
		Ok(())
	}


	fn spawn_pool( &self, fut: Pin<Box< dyn Future< Output = () > + 'static >> ) -> ThesRes<()>
	{
		self.spawner.borrow_mut().spawn_local( fut ).unwrap();
		Ok(())
	}
}
