use crate :: { import::*, * };


thread_local!
(
	static EXEC: OnceCell<Box< dyn Executor >> = OnceCell::INIT;
);



pub fn init( new_exec: Box< dyn Executor > ) -> ThesRes<()>
{
	EXEC.with( move |exec| -> ThesRes<()>
	{
		exec

			.set( new_exec )
			.map_err( |_| -> Error { ThesError::DoubleExecutorInit.into() } )
	})
}



pub fn default_init()
{
	EXEC.with( move |exec|
	{
		if exec.get().is_none()
		{
			init( box super::exec03::Exec03::default() ).unwrap()
		}
	});
}



pub fn spawn_pinned( fut: Pin<Box< dyn Future< Output = () > + 'static >> ) -> ThesRes<()>
{
	EXEC.with( move |exec| -> ThesRes<()>
	{
		default_init();
		exec.get().unwrap().spawn( fut )
	})
}



pub fn spawn( fut: impl Future< Output = () > + 'static ) -> ThesRes<()>
{
	spawn_pinned( fut.boxed() )
}



pub fn run()
{
	EXEC.with( move |exec|
	{
		default_init();
		exec.get().unwrap().run();
	});
}
