use crate :: { import::*, single_thread::* };

pub struct Inbox<A> where A: Actor + 'static
{
	handle: mpsc::UnboundedSender  <Box< dyn Envelope<A>           >> ,
	msgs  : mpsc::UnboundedReceiver<Box< dyn Envelope<A> + 'static >> ,
}

impl<A> Inbox<A> where A: Actor + 'static
{
	pub fn new() -> Self
	{
		let (handle, msgs) = mpsc::unbounded();

		Self { handle, msgs }
	}

	pub fn sender( &self ) -> mpsc::UnboundedSender<Box< dyn Envelope<A> >>
	{
		self.handle.clone()
	}
}


impl<A> Mailbox<A> for Inbox<A> where A: Actor + 'static
{
	fn start( &mut self, mut actor: A ) -> TupleResponse
	{
		async move
		{
			let mut next = await!( self.msgs.next() );

			while next.is_some()
			{
				if let Some( envl ) = next
				{
					await!( envl.handle( &mut actor ) );
					next = await!( self.msgs.next() )
				}

				else { break }
			}

		}.boxed()
	}
}


impl<A> Default for Inbox<A> where A: Actor
{
	fn default() -> Self { Self::new() }
}
