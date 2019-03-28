use crate :: { import::*, single_thread::* };

pub struct Inbox<A>
{
	handle : mpsc::UnboundedSender  < Box< dyn Envelope<A> >>,
}

impl<A> Inbox<A> where A: Actor + 'static
{
	pub fn start( exec: &mut impl Spawn, mut actor: A, mut msgs: mpsc::UnboundedReceiver< Box< dyn Envelope<A> + 'static >> )
	{
		exec.spawn( async move
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

		}).expect( "failed to spawn Mailbox" );

	}
}


impl<A> Mailbox<A> for Inbox<A> where A: Actor + 'static
{
	fn new( actor: A, exec: &mut impl Spawn ) -> Self
	{
		let (handle, msgs) = mpsc::unbounded();

		// let mb =

		// TODO: centralize spawning
		//
		// std::thread::spawn( ||
		// {
		// 	futures::executor::block_on( Self::start( actor, msgs ) );
		// });
		//
		Self::start( exec, actor, msgs );

		Self { handle }
	}

	fn addr<Adr>( &mut self ) -> Adr where Adr: Address<A>
	{
		Adr::new( self.handle.clone() )
	}
}
