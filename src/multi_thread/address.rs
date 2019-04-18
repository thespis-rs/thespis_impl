use crate :: { import::*, multi_thread::* };



pub struct Addr< A: Actor >
{
	mb: mpsc::UnboundedSender<Box< dyn Envelope<A> + Send >>,
}

impl< A: Actor > Clone for Addr<A>
{
	fn clone( &self ) -> Self
	{
		Self { mb: self.mb.clone() }
	}
}

impl<A> Addr<A> where A: Actor
{
	// TODO: take a impl trait instead of a concrete type. This can be fixed once we
	// ditch channels or write some channels that implement sink.
	//
	pub fn new( mb: mpsc::UnboundedSender<Box< dyn Envelope<A> + Send >> ) -> Self
	{
		Self{ mb }
	}
}

impl<A> ThreadSafeAddress<A> for Addr<A>

	where A: Actor,

{
	fn send<M>( &mut self, msg: M ) -> ThreadSafeResponse< ThesRes<()> >

		where  A                    : Handler< M >                ,
		       M                    : Message<Result = ()> + Send ,
		      <M as Message>::Result: Send                        ,

	{
		async move
		{
			let envl: Box< dyn Envelope<A> + Send >= Box::new( SendEnvelope::new( msg ) );

			await!( self.mb.send( envl ) )?;

			Ok(())

		}.boxed()
	}



	// TODO: Why do actor and address have to be static here? The single threaded version doesn't require static here for actor
	//
	fn call<M>( &mut self, msg: M ) -> ThreadSafeResponse< ThesRes<<M as Message>::Result> >

		where  A                    : Handler< M >   ,
		       M                    : Message + Send ,
		      <M as Message>::Result: Send           ,

	{
		let mut mb = self.mb.clone();

		async move
		{
			let (ret_tx, ret_rx) = oneshot::channel::< <M as Message>::Result >();

			let envl: Box< dyn Envelope<A> + Send > = Box::new( CallEnvelope::new( msg, ret_tx ) );

			await!( mb.send( envl ) )?;

			Ok( await!( ret_rx )? )

		}.boxed()
	}



	fn recipient<M>( &self ) -> Box< dyn ThreadSafeRecipient<M> >

		where  M                    : Message + Send  ,
		       A                    : Handler<M>      ,
		      <M as Message>::Result: Send            ,

	{
		box Receiver{ addr: self.clone() }
	}
}



struct Receiver<A: Actor>
{
	addr: Addr<A>
}


impl<A: Actor> Clone for Receiver<A>
{
	fn clone( &self ) -> Self
	{
		Self { addr: self.addr.clone() }
	}
}



impl<A, M> ThreadSafeRecipient<M> for Receiver<A>

	where  A                    : Handler< M >    ,
	       M                    : Message + Send  ,
	      <M as Message>::Result: Send            ,

{
	default fn send( &mut self, msg: M ) -> ThreadSafeResponse< ThesRes<()> >

		where M: Message<Result = ()>,
	{
		self.addr.send( msg )
	}



	default fn call( &mut self, msg: M ) -> ThreadSafeResponse< ThesRes<<M as Message>::Result> >
	{
		self.addr.call( msg )
	}

	fn clone_box( &self ) -> Box< dyn ThreadSafeRecipient<M> >
	{
		box Self { addr: self.addr.clone() }
	}
}
