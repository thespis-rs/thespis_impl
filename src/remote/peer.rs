use { crate :: { import::*, ThesError, runtime::rt, remote::ServiceID, remote::ConnID, single_thread::{ Addr, Rcpnt } } };

mod close_connection;
mod call;
mod incoming;

pub use close_connection :: CloseConnection ;
pub use call             :: Call            ;
    use incoming         :: Incoming        ;


// Reduce trait bound boilerplate, since we have to repeat them all over
//
pub trait BoundsIn <MulService>: 'static + Stream< Item = Result<MulService, Error> > + Unpin {}
pub trait BoundsOut<MulService>: 'static + Sink<MulService, SinkError=Error> + Unpin          {}
pub trait BoundsMulService     : 'static + Message<Result=()> + MultiService                  {}

impl<T, MulService> BoundsIn<MulService> for T
where T: 'static + Stream< Item = Result<MulService, Error> > + Unpin {}

impl<T, MulService> BoundsOut<MulService> for T
where T: 'static + Sink<MulService, SinkError=Error> + Unpin {}

impl<T> BoundsMulService for T
where T: 'static + Message<Result=()> + MultiService {}


/// Represents a connection to another process over which you can send actor messages.
///
/// TODO: - let the user subscribe to connection close event.
///       - if you still have a recipient, so an address to this peer, but the remote end closes,
///         what happens when you send on the recipient (error handling in other words)
///
/// ### Closing the connection
///
/// The reasoning behind a peer is that it is tied to a stream/sink, often a framed connection.
/// When the connection closes for whatever reason, the peer should dissappear and no longer handle
/// any messages.
///
/// If the remote closes the connection, and you are no longer holding any addresses to this
/// peer (or recipients for remote actors), then the peer will get dropped.
///
/// If you do hold recipients and try to send on them, 2 things can happen. Since Send is like
/// throwing a message in a bottle, without feedback, it's infallible, so your message will
/// just get dropped silently. If you use call, which returns a result, you will get an error.
///
/// It's not yet implemented, but there will be an event that you will be able to subscribe to
/// to detect closed connections, so you can drop your recipients, try to reconnect, etc...
///
/// ### Actor shutdown
///
//
pub struct Peer<Out, MulService>

	where Out        : BoundsOut<MulService> ,
	      MulService : BoundsMulService      ,

{
	/// The sink
	//
	outgoing      : Option< Out >,

	/// This is needed so that the loop listening to the incoming stream can send messages to this actor.
	/// The loop runs in parallel of the rest of the actor, yet processing incoming messages need mutable
	/// access to our state, so we have to pass through a message, or we need to put everything in Rc<RefCell>>.
	/// For now, passing messages seems the cleaner solution.
	///
	/// It also allows us to hand out our address to things that have to respond to the remote on our connection.
	//
	addr          : Option< Addr<Self> >,

	/// The handle to the spawned listen function. If we drop this, the listen function immediately stops.
	//
	listen_handle : Option< RemoteHandle<()> >,

	/// Information required to process incoming messages. The first element is a boxed Rcpnt, and the second is
	/// the service map that takes care of this service type.
	//
	services      : HashMap< &'static <MulService as MultiService>::ServiceID , (Box<Any>, Box< dyn ServiceMap<MulService> >) >,

	/// All services that we relay to another peer. It has to be of the same type for now since there is
	/// no trait for peers.
	//
	relay         : HashMap< &'static <MulService as MultiService>::ServiceID, Addr<Self> >,

	/// We use onshot channels to give clients a future that will resolve to their response.
	//
	responses     : HashMap< <MulService as MultiService>::ConnID, oneshot::Sender<MulService> >,
}



impl<Out, MulService> Actor for Peer<Out, MulService>

	where Out        : BoundsOut<MulService> ,
	      MulService : BoundsMulService      ,
{
	// fn started ( &mut self ) -> Response<()>
	// {
	// 	async move
	// 	{
	// 		trace!( "Started Peer actor" );

	// 	}.boxed()
	// }


	// fn stopped ( &mut self ) -> Response<()>
	// {
	// 	async move
	// 	{
	// 		trace!( "Stopped Peer actor" );

	// 	}.boxed()
	// }
}



impl<Out, MulService> Peer<Out, MulService>

	where Out        : BoundsOut<MulService> ,
	      MulService : BoundsMulService      ,

{
	/// Create a new peer to represent a connection to some remote.
	//
	pub fn new( addr: Addr<Self>, incoming: impl BoundsIn<MulService>, outgoing: Out ) -> Self
	{
		let listen = Self::listen( addr.clone(), incoming );

		// Only way to able to close the connection... if it's tokio underneath. Anyways, since
		// we take any kind of sink and stream, it's probably best to specifically stop listening
		// when we want to close rather than count on drop side-effects.
		//
		// https://users.rust-lang.org/t/tokio-tcp-connection-not-closed-when-sender-is-dropped-futures-0-3-compat-layer/26910
		// https://github.com/tokio-rs/tokio/issues/852#issuecomment-459766144
		//
		let (remote, handle) = listen.remote_handle();
		rt::spawn( remote ).expect( "Failed to spawn listener" );

		Self
		{
			outgoing     : Some( outgoing ) ,
			addr         : Some( addr )     ,
			responses    : HashMap::new()   ,
			services     : HashMap::new()   ,
			relay        : HashMap::new()   ,
			listen_handle: Some( handle )   ,
		}
	}


	/// Register a handler for a service that you want to expose over this connection.
	///
	/// TODO: define what has to happen when called several times on the same service
	///       options: 1. error
	///                2. replace prior entry
	///                3. allow several handlers for the same service (not very likely)
	///
	/// TODO: review api design. We take the service as a type parameter yet still require the user to pass in
	///       sid? Should we not require a trait bound on Service rather than on Message? Is that possible
	///       since we need a Recipient to it? We would need service map type if we were to call sid() on the
	///       Service, but maybe we could take an explicit type for the service map?
	///       Currently this requires the user to instantiate a new service map per service. Do we want this?
	///       I think current impl is a zero sized type, so that's probably not problem.
	///       Same questions for relayed services
	//
	pub fn register_service<Service: Message>
	(
		&mut self                                             ,
		     sid    : &'static <MulService as MultiService>::ServiceID ,
		     sm     : Box< dyn ServiceMap<MulService> >       ,
		     handler: Box< dyn Recipient <Service   > >       ,
	)
	{
		self.services.insert( sid, (box Rcpnt::new( handler ), sm) );
	}


	/// Tell this peer to make a given service avaible to a remote, by forwarding incoming requests to the given peer.
	/// For relaying services from other processes.
	//
	pub fn register_relayed_service<Service: Message>( &mut self, sid: &'static <MulService as MultiService>::ServiceID, peer: Addr<Self> )
	{
		self.relay.insert( sid, peer );
	}



	async fn listen( mut self_addr: Addr<Self>, mut incoming: impl BoundsIn<MulService> )
	{
		loop
		{
			let event: Option< Result< MulService, _ > > = await!( incoming.next() );

			trace!( "got incoming event on stream" );

			match event
			{
				Some( conn ) => { match conn
				{
					Ok ( mesg  ) =>
					{
						await!( self_addr.send( Incoming { mesg } ) ).expect( "Send to self in peer" );
					},

					Err( error ) =>
					{
						error!( "Error extracting MultiService from stream: {:#?}", error );

						// TODO: we should send an error to the remote before closing the connection.
						//       we should also close the sending end.
						//
						// return Err( ThesError::CorruptFrame.into() );
					}
				}},

				None =>
				{
					trace!( "Connection closed" );

					return await!( self_addr.send( CloseConnection ) ).expect( "Send Drop to self in Peer" );
				}
			};
		}
	}



	// actually send the message accross the wire
	//
	async fn send_msg( &mut self, msg: MulService ) -> ThesRes<()>
	{
		match &mut self.outgoing
		{
			Some( out ) => await!( out.send( msg ) )                             ,
			None        => Err( ThesError::PeerSendAfterCloseConnection.into() ) ,
		}
	}
}



// On an outgoing call, we need to store the conn_id and the peer to whome to return the response
//
// On outgoing call made by a local actor, store in the oneshot sender in self.responses
//
impl<Out, MulService> Handler<MulService> for Peer<Out, MulService>

	where Out        : BoundsOut<MulService> ,
	      MulService : BoundsMulService      ,

{
	fn handle( &mut self, msg: MulService ) -> Response<()>
	{
		async move
		{
			trace!( "Peer sending OUT" );

			let _ = await!( self.send_msg( msg ) );

		}.boxed()
	}
}



