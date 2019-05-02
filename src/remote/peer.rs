use { crate :: { import::*, ThesError, runtime::rt, remote::{ ServiceID, ConnID, Codecs }, Addr, Receiver } };


mod close_connection  ;
mod connection_error  ;
mod peer_event        ;
mod call              ;
mod incoming          ;
mod register_relay    ;

pub use call              :: Call             ;
pub use close_connection  :: CloseConnection  ;
pub use connection_error  :: ConnectionError  ;
pub use peer_event        :: PeerEvent        ;
pub use register_relay    :: RegisterRelay    ;
    use incoming          :: Incoming         ;
    use peer_event        :: RelayEvent       ;

// Reduce trait bound boilerplate, since we have to repeat them all over
//
pub trait BoundsIn <MS>: 'static + Stream< Item = Result<MS, Error> > + Unpin {}
pub trait BoundsOut<MS>: 'static + Sink<MS, SinkError=Error> + Unpin + Send {}
pub trait BoundsMS     : 'static + Message<Return=()> + MultiService + Send {}

impl<T, MS> BoundsIn<MS> for T
where T: 'static + Stream< Item = Result<MS, Error> > + Unpin {}

impl<T, MS> BoundsOut<MS> for T
where T: 'static + Sink<MS, SinkError=Error> + Unpin + Send {}

impl<T> BoundsMS for T
where T: 'static + Message<Return=()> + MultiService {}


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
pub struct Peer<Out, MS>

	where Out: BoundsOut<MS> ,
	      MS : BoundsMS      ,

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
	listen_handle : Option< RemoteHandle<()>>,

	/// Information required to process incoming messages. The first element is a boxed Receiver, and the second is
	/// the service map that takes care of this service type.
	//
	services      : HashMap<&'static <MS as MultiService>::ServiceID ,(BoxAny, BoxServiceMap<MS>)>,

	/// All services that we relay to another peer. It has to be of the same type for now since there is
	/// no trait for peers.
	///
	/// We store a map of the sid to the actor_id and then a map from actor_id to both addr and
	/// remote handle for the PeerEvents.
	///
	/// These two fields should be kept in sync. Eg, we call unwrap on the get_mut on relays if
	/// we found the id in relayed.
	//
	relayed       : HashMap< &'static <MS as MultiService>::ServiceID, usize >,
	relays        : HashMap< usize, (Addr<Self>, RemoteHandle<()>)           >,

	/// We use onshot channels to give clients a future that will resolve to their response.
	//
	responses     : HashMap< <MS as MultiService>::ConnID, oneshot::Sender<MS> >,

	/// The pharos allows us to have observers.
	//
	pharos        : Pharos<PeerEvent>,
}



impl<Out, MS> Actor for Peer<Out, MS>

	where Out        : BoundsOut<MS> ,
	      MS : BoundsMS      ,
{
	// fn started ( &mut self ) -> Return<()>
	// {
	// 	async move
	// 	{
	// 		trace!( "Started Peer actor" );

	// 	}.boxed()
	// }


	// fn stopped ( &mut self ) -> Return<()>
	// {
	// 	async move
	// 	{
	// 		trace!( "Stopped Peer actor" );

	// 	}.boxed()
	// }
}



impl<Out, MS> Peer<Out, MS>

	where Out: BoundsOut<MS> ,
	      MS : BoundsMS      ,

{
	/// Create a new peer to represent a connection to some remote.
	//
	pub fn new( addr: Addr<Self>, incoming: impl BoundsIn<MS>, outgoing: Out ) -> ThesRes< Self >
	{
		trace!( "create peer" );

		// Hook up the incoming stream to our address.
		//
		let mut addr2 = addr.clone();

		let listen = async move
		{
			// We need to map this to a custom type, since we had to impl Message for it.
			//
			let stream  = &mut incoming.map( |msg| Incoming{ msg } );

			// This can fail if:
			// - channel is full (TODO: currently we use unbounded, so that won't happen, but it might
			//   use unbounded amounts of memory.)
			// - the receiver is dropped. The receiver is our mailbox, so it should never be dropped
			//   as long as we have an address to it.
			//
			// So, I think we can unwrap for now.
			//
			await!( addr2.send_all( stream ) ).expect( "peer send to self");

			// Same as above.
			//
			await!( addr2.send( CloseConnection{ remote: true } ) ).expect( "peer send to self");
		};

		// When we need to stop listening, we have to drop this future, because it contains
		// our address, and we won't be dropped as long as there are adresses around.
		//
		let (remote, handle) = listen.remote_handle();
		rt::spawn( remote )?;

		Ok( Self
		{
			outgoing     : Some( outgoing ) ,
			addr         : Some( addr )     ,
			responses    : HashMap::new()   ,
			services     : HashMap::new()   ,
			relayed      : HashMap::new()   ,
			relays       : HashMap::new()   ,
			listen_handle: Some( handle )   ,
			pharos       : Pharos::new()    ,
		})
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
		&mut self                                              ,
		     sid    : &'static <MS as MultiService>::ServiceID ,
		     sm     : BoxServiceMap<MS>                        ,
		     handler: BoxRecipient<Service>                    ,
	)
	{
		self.services.insert( sid, (box Receiver::new( handler ), sm) );
	}



	// actually send the message accross the wire
	//
	async fn send_msg( &mut self, msg: MS ) -> ThesRes<()>
	{
		match &mut self.outgoing
		{
			Some( out ) => await!( out.send( msg ) )                             ,
			None        => Err( ThesError::PeerSendAfterCloseConnection.into() ) ,
		}
	}



	// actually send the message accross the wire
	//
	fn prep_error( cid: <MS as MultiService>::ConnID, err: ConnectionError ) -> MS
	{
		// it's important that the sid is null here, because a response with both cid and sid
		// not null is interpreted as an error.
		//
		let serialized   = serde_cbor::to_vec( &err ).expect( "serialize response" );
		let codec: Bytes = Codecs::CBOR.into();

		let codec2 = match <MS as MultiService>::CodecAlg::try_from( codec )
		{
			Ok ( c ) => c,
			Err( _ ) => panic!( "Failed to create codec from bytes" ),
		};

		MS::create
		(
			<MS as MultiService>::ServiceID::null(),
			cid,
			codec2,
			serialized.into()
		)
	}
}



// On an outgoing call, we need to store the conn_id and the peer to whome to return the response
//
// On outgoing call made by a local actor, store in the oneshot sender in self.responses
//
impl<Out, MS> Handler<MS> for Peer<Out, MS>

	where Out: BoundsOut<MS> ,
	      MS : BoundsMS      ,

{
	fn handle( &mut self, msg: MS ) -> ReturnNoSend<()>
	{
		Box::pin( async move
		{
			trace!( "Peer sending OUT" );

			let _ = await!( self.send_msg( msg ) );

		})
	}
}


impl<Out, MS> Observable<PeerEvent> for Peer<Out, MS>

	where Out: BoundsOut<MS> ,
	      MS : BoundsMS      ,
{

	/// Register an observer to receive events from this connection. This will allow you to detect
	/// Connection errors and loss. Note that the peer automatically goes in shut down mode if the
	/// connection is closed. When that happens, you should drop all remaining addresses of this peer.
	/// An actor does not get dropped as long as you have adresses to it.
	///
	/// You can then create a new connection, frame it, and create a new peer. This will send you
	/// a PeerEvent::ConnectionClosed if the peer is in unsalvagable state and you should drop all addresses
	///
	/// See [PeerEvent] for more details on all possible events.
	//
	fn observe( &mut self, queue_size: usize ) -> mpsc::Receiver<PeerEvent>
	{
		self.pharos.observe( queue_size )
	}
}




