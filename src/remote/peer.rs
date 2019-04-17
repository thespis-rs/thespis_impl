use { crate :: { import::*, ThesError, runtime::rt, remote::ServiceID, remote::ConnID, single_thread::Addr } };

pub trait BoundsIn <MulService>: 'static + Stream< Item = Result<MulService, Error> > + Unpin {}
pub trait BoundsOut<MulService>: 'static + Sink<MulService, SinkError=Error> + Unpin          {}
pub trait BoundsMulService     : 'static + Message<Result=()> + MultiService                  {}

impl<T, MulService> BoundsIn<MulService> for T
where T: 'static + Stream< Item = Result<MulService, Error> > + Unpin {}

impl<T, MulService> BoundsOut<MulService> for T
where T: 'static + Sink<MulService, SinkError=Error> + Unpin {}

impl<T> BoundsMulService for T
where T: 'static + Message<Result=()> + MultiService {}


pub struct Peer<Out, MulService>

	where Out        : BoundsOut<MulService> ,
	      MulService : BoundsMulService      ,
{
	outgoing: Out                     ,
	p       : PhantomData<MulService> ,
	addr    : Addr<Self>              ,

	services   : HashMap< <MulService as MultiService>::ServiceID , Box< Any >   > ,
	local_sm   : HashMap< <MulService as MultiService>::ServiceID , Box< dyn ServiceMap<MulService>>> ,
	relay      : HashMap< <MulService as MultiService>::ServiceID , Box< dyn Recipient<Any> >   > ,
	responses  : HashMap< <MulService as MultiService>::ConnID    , oneshot::Sender<MulService> > ,
	connections: HashMap< <MulService as MultiService>::ConnID    , String                      > ,

}



impl<Out, MulService> Actor for Peer<Out, MulService>

	where Out        : BoundsOut<MulService> ,
	      MulService : BoundsMulService      ,
{
	fn started ( &mut self ) -> TupleResponse
	{
		async move
		{
			trace!( "Started Peer actor" );

		}.boxed()
	}


	fn stopped ( &mut self ) -> TupleResponse
	{
		async move
		{
			trace!( "Stopped Peer actor" );

		}.boxed()
	}
}



impl<Out, MulService> Peer<Out, MulService>

	where Out        : BoundsOut<MulService> ,
	      MulService : BoundsMulService      ,
{
	pub fn new( addr: Addr<Self>, incoming: impl BoundsIn<MulService>, outgoing: Out /*, _sm: impl ServiceMap*/ ) -> Self
	{
		rt::spawn( Self::listen( addr.clone(), incoming ) ).expect( "Failed to spawn listener" );

		Self
		{
			outgoing                    ,
			addr                        ,
			p          : PhantomData    ,
			responses  : HashMap::new() ,
			services   : HashMap::new() ,
			local_sm   : HashMap::new() ,
			relay      : HashMap::new() ,
			connections: HashMap::new() ,
		}
	}


	pub fn register_service( &mut self, sid: <MulService as MultiService>::ServiceID, sm: Box<dyn ServiceMap<MulService>>, handler: Box< dyn Any > )
	{
		self.services.insert( sid.clone(), handler );
		self.local_sm.insert( sid        , sm      );
	}



	async fn listen( mut self_addr: Addr<Self>, mut incoming: impl BoundsIn<MulService> )
	{
		loop
		{
			let event: Option< Result< MulService, _ > > = await!( incoming.next() );

			dbg!( "got incoming event on stream" );

			match event
			{
				Some( conn ) =>
				{
					match conn
					{
						Ok ( mesg  ) =>
						{
							dbg!( &mesg.conn_id() );
							await!( self_addr.send( Incoming { mesg } ) ).expect( "Send to self in peer" );
							dbg!( "after send incoming in peer" );
						},

						Err( error ) =>
						{
							error!( "Error extracting MultiService from stream: {:#?}", error );

							// TODO: we should send an error to the remote before closing the connection.
							//       we should also close the sending end.
							//
							// return Err( ThesError::CorruptFrame.into() );
						}
					}
				},

				None => { trace!( "Connection closed" ); return } // Ok(())    // Disconnected
			};
		}
	}



	// actor in self.process => deserialize, use send on recipient
	// actor in self.relay   => forward
	// actor unknown         => send back and log error
	//
	async fn handle_send( &mut self, _frame: MulService )
	{

	}


	// actor in self.process => deserialize, use call on recipient,
	//                          when completes, reconstruct multiservice with connID for response.
	// actor in self.relay   => Create Call and send to recipient found in self.relay.
	// actor unknown         => send back and log error
	//
	async fn handle_call( &mut self, _frame: MulService )
	{

	}


	// actor in self.process => look in self.responses, deserialize and send response in channel.
	// actor in self.relay   => Create Call and send to recipient found in self.relay.
	// actor unknown         => send back and log error
	//
	async fn handle_response( &mut self, _frame: MulService )
	{

	}


	// log error?
	// allow user to set a handler for these...
	//
	async fn handle_error( &mut self, _frame: MulService )
	{

	}


	// actually send the message accross the wire
	//
	async fn send_msg( &mut self, msg: MulService )
	{
		match await!( self.outgoing.send( msg ) )
		{
			Ok (_) => { trace!( "Peer: successfully wrote to stream"       ); },
			Err(e) => { error!( "Peer: failed to write to stream: {:?}", e ); },
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
			dbg!( "Peer sending OUT" );

			await!( self.send_msg( msg ) );

		}.boxed()
	}
}






pub struct ForwardCall<MulService: MultiService>
{
	reply_to: Box< dyn Recipient<MulService> >,
	mesg    : MulService,
}



/// Type representing the outgoing call
//
pub struct Call<MulService: MultiService>
{
	mesg: MulService,
}

impl<MulService: 'static +  MultiService> Message for Call<MulService>
{
	type Result = oneshot::Receiver<MulService>;
}

impl<MulService: MultiService> Call<MulService>
{
	pub fn new( mesg: MulService ) -> Self
	{
		Self{ mesg }
	}
}



/// Type representing Messages coming in over the wire
//
struct Incoming<MulService: MultiService>
{
	pub mesg: MulService,
}

impl<MulService: 'static + MultiService> Message for Incoming<MulService>
{
	type Result = ();
}



/// Handler for outgoing Calls
//
impl<Out, MulService> Handler<Call<MulService>> for Peer<Out, MulService>

	where Out: BoundsOut<MulService>,
	      MulService: BoundsMulService,
{
	fn handle( &mut self, call: Call<MulService> ) -> Response< <Call<MulService> as Message>::Result >
	{
		dbg!( "peer: starting call handler" );

		let (sender, receiver) = oneshot::channel::< MulService >();

		let conn_id = call.mesg.conn_id().expect( "Failed to get connection ID from call" );

		self.responses.insert( conn_id, sender );

		let fut = async move
		{
			await!( self.send_msg( call.mesg ) );

			// We run expect here, because we are holding the sender part of this, so
			// it should never be cancelled. It is more pleasant for the client
			// not to have to deal with 2 nested Results.
			//
			// TODO: There is one exeption, when this actor get's stopped, the client will
			// be holding this future and the expect will panic. That's bad, but we will
			// investigate how to deal with that once we have some more real life usecases.
			//
			// TODO: 2: We return a oneshot::Receiver, rather than a future, so we expose
			// implementation details to the user = BAD!!!
			//
			receiver

		}.boxed();

		dbg!( "peer: returning from call handler" );

		fut
	}
}



/// Handler for incoming messages
//
impl<Out, MulService> Handler<Incoming<MulService>> for Peer<Out, MulService>

	where Out       : BoundsOut<MulService>,
	      MulService: BoundsMulService     ,
{
	fn handle( &mut self, incoming: Incoming<MulService> ) -> Response<()>
	{
		dbg!( "Incoming message!" );

		async move
		{
			let frame = incoming.mesg;

			// algorithm for incoming messages. Options are:
			//
			// 1. incoming send/call               for local/relayed/unknown actor (6 options)
			// 2.       response to outgoing call from local/relayed actor         (2 options)
			// 3. error response to outgoing call from local/relayed actor         (2 options)
			//
			// 4 possibilities with ServiceID and ConnID. These can be augmented with
			// predicates about our local state (sid in local table, routing table, unknown), + the codec
			// which gives us largely the 10 needed states:
			//
			// SID  present -> always tells us if it's for local/relayed/unknown actor
			//                 based on our routing tables
			//
			//                 if it's absent, it can come from our open connections table.
			//
			// (leaves distinguishing between send/call/response/error)
			//
			// CID   absent  -> Send
			// CID   unknown -> Call
			//
			// CID   present -> Response/Error
			//
			// (leaves Response/Error)
			//
			// both  present -> Error, if response, no need for SID since ConnID will be in our open connections table
			// none  present -> not supported?
			// codec present -> obligatory, not used to distinguish use case, but do we accept utf8 for errors?
			//                  maybe not, strongly typed errors defined by thespis encoded with cbor seem better.
			//
			dbg!( &frame.conn_id().unwrap() );
			dbg!( &frame.service().unwrap() );
			dbg!( &frame.service().unwrap().is_null() );

			let sid = frame.service().expect( "fail to getting conn_id from frame"    );
			let cid = frame.conn_id().expect( "fail to getting service_id from frame" );

			// it's an incoming send
			//
			if cid.is_null()
			{
				trace!( "Incoming Send" );

				await!( self.handle_send( frame ) );
			}

			// it's a call
			//
			else if !self.responses.contains_key( &cid ) // && !self.relayed_calls.contains_key( &cid )
			{
				trace!( "Incoming Call" );

				// service_id in self.process => deserialize, use call on recipient, when completes,
				//                               reconstruct multiservice with connID for response.
				//
				if let Some( handler ) = self.services.remove( &sid )
				{
					trace!( "Incoming Call for local Actor" );

					// Call actor
					//
					let sm      = self.local_sm.get( &sid ).expect( "failed to find service map." );
					let addr    = self.addr.clone();
					let service = handler;//.clone();

					await!( sm.call_service( frame, service, addr.recipient::<MulService>() ) );
				}


				// service_id in self.relay   => Create Call and send to recipient found in self.relay.
				//
				else if let Some( _r ) = self.relay.get( &sid )
				{
					trace!( "Incoming Call for relayed Actor" );

				}

				// service_id unknown         => send back and log error
				//
				else
				{
					trace!( "Incoming Call for unknown Actor" );

				}


			}

			// it's a response
			//
			else if let Some( channel ) = self.responses.remove( &cid )
			{
				trace!( "Incoming Response" );

				// TODO: verify our error handling story here. Normally if this
				// fails it means the receiver of the channel was dropped... so
				// they are no longer interested in the reponse? Should we log?
				// Have a different behaviour in release than debug?
				//
				let _ = channel.send( frame );
			}

			// it's a response for a relayed actor (need to keep track of relayed cid's)
			//
			// else if let Some( channel = self.responses.get( cid ))
			// {
			// 	trace!( "Incoming Response" );

			// }

			// it's an error
			//
			else
			{
				await!( self.handle_error( frame ) )
			}

		}.boxed()
	}
}
