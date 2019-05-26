use
{
	crate                :: { import::*                } ,
	wasm_bindgen_futures :: { futures_0_3::spawn_local } ,
};


/// An executor that works on WASM.
//
pub struct WasmRT
{
	spawned: Rc<RefCell<Vec< Pin<Box< dyn Future< Output = () > + 'static >>>>> ,
	running: RefCell<bool>                                                      ,
}



impl Default for WasmRT
{
	fn default() -> Self
	{
		WasmRT
		{
			spawned: Rc::new( RefCell::new( vec![] ) ),
			running: RefCell::new( false )            ,
		}
	}
}



impl Executor for WasmRT
{
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
				spawn_local( fut );
			}
		};

		{ *self.running.borrow_mut() = true; }

		spawn_local( task );
	}


	/// Spawn a future to be run on the LocalPool (current thread)
	//
	fn spawn( &self, fut: Pin<Box< dyn Future< Output = () > + 'static >> ) -> ThesRes<()>
	{
		if *self.running.borrow()
		{
			spawn_local( fut );
		}

		else
		{
			self.spawned.borrow_mut().push( fut );
		}

		Ok(())
	}


	/// The Executor trait requires this for now, but wasm doesn't have threads yet!
	//
	fn spawn_pool( &self, _fut: Pin<Box< dyn Future< Output = () > + 'static >> ) -> ThesRes<()>
	{
		unreachable!()
	}
}
