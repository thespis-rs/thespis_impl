use crate :: { import::*, * };

pub struct ProcLocalMb<A>
{
	handle : mpsc::UnboundedSender  < Box< dyn Envelope<A> >>,
}

impl<A> ProcLocalMb<A> where A: Actor
{
	async fn start( mut actor: A, mut msgs: mpsc::UnboundedReceiver< Box< dyn Envelope<A> + 'static >> )
	{
		let mut next = await!( msgs.next() );

		while next.is_some()
		{
			if let Some( envl ) = next
			{
				await!( envl.handle( &mut actor ) );
				next = await!( msgs.next() )
			}

			else { break }
		}
	}
}


impl<A> Mailbox<A> for ProcLocalMb<A> where A: Actor + 'static
{
	fn new( actor: A ) -> Self
	{
		let (handle, msgs) = mpsc::unbounded();

		let mb = Self { handle };

		use futures::executor::ThreadPool;
		use futures::task::SpawnExt;

		let mut executor = ThreadPool::new().unwrap();

		executor.spawn( Self::start( actor, msgs ) ).unwrap();


		mb
	}

	fn addr<Adr>( &mut self ) -> Adr where Adr: Address<A>
	{
		Adr::new( self.handle.clone() )
	}
}
