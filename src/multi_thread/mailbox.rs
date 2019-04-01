use crate :: { import::*, multi_thread::* };

pub struct Inbox<A> where A: Actor
{
	handle: mpsc::UnboundedSender  <Box< dyn Envelope<A> + Send >> ,
	msgs  : mpsc::UnboundedReceiver<Box< dyn Envelope<A> + Send >> ,
}

impl<A> Inbox<A> where A: Actor
{
	pub fn new() -> Self
	{
		let (handle, msgs) = mpsc::unbounded();

		Self { handle, msgs }
	}

	pub fn sender( &self ) -> mpsc::UnboundedSender<Box< dyn Envelope<A> + Send >>
	{
		self.handle.clone()
	}
}



impl<A> Mailbox<A> for Inbox<A> where A: Actor
{
	fn start( self, mut actor: A ) -> Pin<Box< dyn Future<Output=()> >> { async move
	{
		// We need to drop the handle, otherwise the channel will never close and the program will not
		// terminate. Like this when the user drops all their handles, this loop will automatically break.
		//
		let Inbox{ mut msgs, handle } = self;
		drop( handle );

		trace!( "starting loop in mailbox" );

		loop
		{
			match await!( msgs.next() )
			{
				Some( envl ) => { await!( envl.handle( &mut actor ) ); }
				None         => { break;                               }
			}
		}

	}.boxed() }
}



impl<A> Default for Inbox<A> where A: Actor
{
	fn default() -> Self { Self::new() }
}
