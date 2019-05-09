use crate :: { import::*, ThesErr, runtime::rt, remote::{ self, error::*, ServiceID, ConnID, Codecs }, Addr, Receiver };


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
pub trait BoundsIn <MS: BoundsMS>: 'static + Stream< Item = Result<MS, <MS as MultiService>::Error> > + Unpin {}
pub trait BoundsOut<MS: BoundsMS>: 'static + Sink<MS, SinkError=ThesRemoteErr > + Unpin + Send {}
pub trait BoundsMS               : 'static + Message<Return=()> + MultiService< Error=ThesRemoteErr > + Send + fmt::Debug {}

impl<T, MS> BoundsIn<MS> for T

	where T : 'static + Stream< Item = Result<MS, <MS as MultiService>::Error> > + Unpin,
   	   MS: BoundsMS
{}

impl<T, MS> BoundsOut<MS> for T

	where T : 'static + Sink<MS, SinkError=ThesRemoteErr > + Unpin + Send,
	      MS: BoundsMS
{}

impl<T> BoundsMS for T
where T: 'static + Message<Return=()> + MultiService< Error=ThesRemoteErr > + Send + fmt::Debug {}


/// Represents a connection to another process over which you can send actor messages.
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
/// just get dropped silently. If you use call, which returns a result, you will get an error
/// (ThesError::PeerSendAfterCloseConnection).
///
/// Peer uses the pharos crate to be observable over [`PeerEvent`]. This allows you to detect
/// when errors happen and to react accordingly. If the connection gets closed, you can make
/// reconnect and make a new peer.
///
//
#[ derive( Actor ) ]
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
	// The error type here needs to correspond to the error type of the recipient we are going to pass
	// to `Servicemap::call_service`. TODO: In principle we should be generic over recipient type, but for now
	// I have put ThesErr, because it's getting to complex.
	//
	services      : HashMap<&'static <MS as MultiService>::ServiceID ,(BoxAny, BoxServiceMap<MS, ThesErr>)>,

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



impl<Out, MS> Peer<Out, MS>

	where Out: BoundsOut<MS> ,
	      MS : BoundsMS      ,

{
	/// Create a new peer to represent a connection to some remote.
	//
	pub fn new( addr: Addr<Self>, incoming: impl BoundsIn<MS>, outgoing: Out ) -> Result< Self, ThesRemoteErr >
	{
		trace!( "create peer" );

		// Hook up the incoming stream to our address.
		//
		let mut addr2 = addr.clone();

		let listen = async move
		{
			// We need to map this to a custom type, since we had to impl Message for it.
			//
			let stream  = &mut incoming.map( |msg|
			{
				Incoming{ msg }
			});

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
		rt::spawn( remote ).context( ThesRemoteErrKind::Spawn{ context: "Incoming stream for peer".into() } )?;

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
	/// ----- Currently the existing entry is replaced with the new one. For the moment
	///       the method is infallible which could no longer be the same if we change this.
	///
	/// TODO: review api design. Currently this needs to be called on a peer, which needs
	///       it's own address, so there is no way to sugar this up. Users will need to
	///       make and address and manual mailbox, then feed the address to peer, then
	///       register services. So it's not really possible to make a reusable method
	///       which takes a socket address, connects and returns an address to a peer, because
	///       you will need to start the mailbox after registering here. Since this method
	///       is...
	///
	/// ----- Actually, if the servicemap could take a peer and register all it's services
	///       that would be awesome. Or a peer could take a servicemap and register all services...
	///       Actuall, that won't be so easy. The user decides which actor handles which
	///       service... We would have to take a vector of (sid, Box<Any>)... not very
	///       clean.
	///
	/// TODO: For now we have put the error type to ThesErr fixed. For usability, we probably
	///       should be generic over that error type.
	//
	pub fn register_service<S, NS>( &mut self, handler: Receiver<S> )

		where  S                    : Service<NS, UniqueID=<MS as MultiService>::ServiceID>,
		      <S as Message>::Return: Serialize + DeserializeOwned                         ,
		       NS                   : ServiceMap<MS, ThesErr> + Send + Sync + 'static    ,

	{
		self.services.insert( <S as Service<NS>>::sid(), (box handler, NS::boxed()) );
	}



	/// Tell this peer to make a given service avaible to a remote, by forwarding incoming requests to the given peer.
	/// For relaying services from other processes.
	//
	pub fn register_relayed_services
	(
		&mut self                                                        ,
		     services    : Vec<&'static <MS as MultiService>::ServiceID> ,
		     peer        : Addr<Self>                                    ,
		     peer_events : mpsc::Receiver<PeerEvent>                     ,

	) -> Result<(), ThesRemoteErr>

	{
		trace!( "peer: starting Handler<RegisterRelay<Out, MS>>" );

		// When called from a RegisterRelay message, it's possible that in the mean time
		// the connection closed. We should immediately return a ConnectionClosed error.
		//
		let mut self_addr = match &self.addr
		{
			Some( self_addr ) => self_addr.clone() ,
			None              =>

				Err( ThesRemoteErrKind::ConnectionClosed{ operation: "register_relayed_services".to_string() } )?,
		};

		let peer_id = < Addr<Self> as Recipient<RelayEvent> >::actor_id( &peer );


		let listen = async move
		{
			// We need to map this to a custom type, since we had to impl Message for it.
			//
			let stream = &mut peer_events.map( |evt| RelayEvent{ id: peer_id, evt } );

			// This can fail if:
			// - channel is full (for now we use unbounded)
			// - the receiver is dropped. The receiver is our mailbox, so it should never be dropped
			//   as long as we have an address to it.
			//
			// So, I think we can unwrap for now.
			//
			await!( self_addr.send_all( stream ) ).expect( "peer send to self" );

			// Same as above.
			// Normally relays shouldn't just dissappear, without notifying us, but it could
			// happen for example that the peer already shut down and the above stream was already
			// finished, we would immediately be here, so we do need to clean up.
			// Since we are doing multi threading it's possible to receive the peers address,
			// but it's no longer valid. So send ourselves a message.
			//
			let evt = PeerEvent::Closed;

			await!( self_addr.send( RelayEvent{ id: peer_id, evt } ) ).expect( "peer send to self");
		};

		// When we need to stop listening, we have to drop this future, because it contains
		// our address, and we won't be dropped as long as there are adresses around.
		//
		let (remote, handle) = listen.remote_handle();
		rt::spawn( remote ).map_err( |_|
		{
			ThesRemoteErrKind::Spawn { context: "Stream of events from relay peer".to_string() }
		})?;

		self.relays .insert( peer_id, (peer, handle) );

		for sid in services
		{
			trace!( "Register relaying to: {}", sid );
			self.relayed.insert( sid, peer_id );
		}

		Ok(())
	}



	// actually send the message accross the wire
	//
	async fn send_msg( &mut self, msg: MS ) -> Result<(), ThesRemoteErr>
	{
		match &mut self.outgoing
		{
			Some( out ) => await!( out.send( msg ) ),

			None =>
			{
				Err( ThesRemoteErrKind::ConnectionClosed{ operation: "send MS".to_string() } )?
			}
		}
	}



	// actually send the error accross the wire. This is for when errors happen on receiving
	// messages (eg. Deserialization errors).
	//
	async fn send_err<'a>
	(
		&'a mut self                                ,
		     cid  : <MS as MultiService>::ConnID ,
		     err  : &'a ConnectionError             ,

		     // whether the connection should be closed (eg stream corrupted)
		     //
		     close: bool                         ,
	)
	{
		if let Some( ref mut out ) = self.outgoing
		{
			let msg = Self::prep_error( cid, err );

			let _ = await!( out.send( msg ) );

			if close {
			if let Some( ref mut addr ) = self.addr
			{
				// until we have bounded channels, this should never fail, so I'm leaving the expect.
				//
				await!( addr.send( CloseConnection{ remote: false } ) ).expect( "send close connection" );
			}}
		}
	}



	// serialize a ConnectionError to be sent across the wire.
	//
	fn prep_error( cid: <MS as MultiService>::ConnID, err: &ConnectionError ) -> MS
	{
		let serialized   = serde_cbor::to_vec( err ).expect( "serialize response" );
		let codec: Bytes = Codecs::CBOR.into();

		let codec2 = match <MS as MultiService>::CodecAlg::try_from( codec )
		{
			Ok ( c ) => c,
			Err( _ ) => panic!( "Failed to create codec from bytes" ),
		};

		// sid null is the marker that this is an error message.
		//
		MS::create
		(
			<MS as MultiService>::ServiceID::null() ,
			cid                                     ,
			codec2                                  ,
			serialized.into()                       ,
		)
	}
}



// Put an outgoing multiservice message on the wire.
// TODO: why do we not return the error?
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



// Pharos, shine!
//
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




