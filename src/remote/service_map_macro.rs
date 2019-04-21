use crate::{ * };


/// This is a beefy macro which is your main interface to using the remote actors. It's unavoidable to
/// require code in the client application because thespis does not know the types of messages you will
/// create, yet we aim at making the difference between local and remote actors seamless for using code.
///
/// This macro will allow deserializing messages to the correct types, as well as creating recipients for
/// remote actors.
///
/// I have named the parameters for clarity, however it's a macro, and not real named parameters so the
/// order needs to be exact.
///
/// Please open declaration section to see the parameter documentation. The module [remote] has more
/// documentation on using remote actors, and there is a remote example and unit tests in the repository
/// which you can look at for a more fully fledged example.
///
/// TODO: document the types made available to the user by this macro.
//
#[ macro_export ]
//
macro_rules! service_map
{

(
	/// namespace unique to this servicemap. It allows you to use your services with several service_maps
	/// and it also gets used in the unique ID generation of services, so different processes can expose
	/// services based on the same type which shall be uniquely identifiable.
	///
	/// A process wanting to send messages needs to create the service map with the same namespace as the
	/// receiving process.
	//
	namespace: $module: ident;

	/// The type used for [Peer]. This is because [Peer] is generic, thus you need to specify the exact type.
	//
	peer_type: $peer_type: ty;

	/// The type you want to use that implements [thespis::MultiService] (the wire format).
	//
	multi_service: $ms_type: ty;

	/// Comma separated list of Services you want to include that can be both Send to and Called. These
	/// must implement thespis::Message with `type Result = ();`. Otherwise send is not possible. There
	/// can not be any overlap with the list `call_only`, otherwise it won't compile.
	//
	send_and_call: $($send_and_call: ident),* $(,)?;

	/// Comma separated list of Services that don't have a tuple as their result type, and thus can be
	/// called but not send to.
	//
	call_only: $($call_only: ident),* $(,)?;

) =>

{

pub mod $module
{

use
{
	// This is actually necessary to see the imports from the module that includes us, otherwise
	// the types the use passes in won't be available.
	// TODO: does this work with paths being passed to the macro rather than types that are in scope
	//       where the macro gets called?
	//
	super          :: { *                                            },
	::futures      :: { future::FutureExt                            },
	::thespis      :: { *                                            },
	::thespis_impl :: { *, remote::*, single_thread::*, runtime::rt  },
	::serde_cbor   :: { self, from_slice as des                      },
	::serde        :: { Serialize, Deserialize, de::DeserializeOwned },
	::std          :: { any::Any                                     },
};

/// Mark the services part of this particular service map, so we can write generic impls only for them.
//
pub trait MarkServices {}

$(	impl MarkServices for $send_and_call {} )*
$(	impl MarkServices for $call_only {} )*


/// The actual service map.
/// Use it to get a recipient to a remote service.
///
#[ derive( Clone ) ]
//
pub struct Services;

impl Services
{
	/// Creates a recipient to a Service type for a remote actor, which can be used in exactly the
	/// same way as if the actor was local.
	//
	pub fn recipient<S>( peer: Addr<$peer_type> ) -> impl Recipient<S>

	where  S                    : MarkServices + Service<UniqueID=ServiceID> + Serialize + DeserializeOwned ,
	      <S as Message>::Result: Serialize + DeserializeOwned                                              ,
	{
		ServicesRecipient::new( peer )
	}



	// Helper function for send_service below
	//
	fn send_service_gen<S>( msg: $ms_type, receiver: &Box< dyn Any > )

		where S                     : Service   + Message<Result=()>,
		      <S as Message>::Result: Serialize + DeserializeOwned  ,

	{
		let     backup: &Rcpnt<S> = receiver.downcast_ref().expect( "downcast_ref failed" );
		let mut rec               = backup.clone_box();

		let message = serde_cbor::from_slice( &msg.mesg() ).expect( "deserialize serviceA" );

		rt::spawn( async move
		{
			await!( rec.send( message ) ).expect( "call actor" );

		}).expect( "spawn call for service" );
	}



	// Helper function for call_service below
	//
	fn call_service_gen<S>
	(
		     msg        :  $ms_type                        ,
		     receiver   : &Box< dyn Any >                  ,
		 mut return_addr:  Box< dyn Recipient< $ms_type >> ,

	)

	where S                     : Service                     ,
	      <S as Message>::Result: Serialize + DeserializeOwned,
	{
		let     backup: &Rcpnt<S> = receiver.downcast_ref().expect( "downcast_ref failed" );
		let mut rec               = backup.clone_box();

		let message = serde_cbor::from_slice( &msg.mesg() ).expect( "deserialize serviceA" );

		rt::spawn( async move
		{
			let resp       = await!( rec.call( message ) ).expect( "call actor" );

			// it's important that the sid is null here, because a response with both cid and sid
			// not null is interpreted as an error.
			//
			let sid        = ServiceID::null();
			let cid        = msg.conn_id().expect( "get conn_id" );
			let serialized = serde_cbor::to_vec( &resp ).expect( "serialize response" );

			let mul        = MultiServiceImpl::new( sid, cid, Codecs::CBOR, serialized.into() );

			await!( return_addr.send( mul ) ).expect( "send response back to peer" );

		}).expect( "spawn call for service" );
	}
}



impl ServiceMap<$ms_type> for Services
{
	// Will match the type of the service id to deserialize the message and send it to the handling actor
	//
	fn send_service( &self, msg: $ms_type, receiver: &Box< dyn Any > )
	{
		let x = msg.service().expect( "get service" );

		match x
		{
			$( _ if x == <$send_and_call>::uid( stringify!( $module ).as_bytes() ) =>

				Self::send_service_gen::<$send_and_call>( msg, receiver ) , )*

			_ => panic!( "got wrong service: {:?}", x ),
		}
	}



	// Will match the type of the service id to deserialize the message and call the handling actor
	//
	fn call_service
	(
		&self                                          ,
		 msg        :  $ms_type                        ,
		 receiver   : &Box< dyn Any >                  ,
		 return_addr:  Box< dyn Recipient< $ms_type >> ,

	)
	{
		let x = msg.service().expect( "get service" );

		match x
		{

			$( _ if x == <$send_and_call>::uid( stringify!( $module ).as_bytes() ) =>

				Self::call_service_gen::<$send_and_call>( msg, receiver, return_addr ) , )*


			$( _ if x == <$call_only>::uid( stringify!( $module ).as_bytes() ) =>

				Self::call_service_gen::<$call_only>( msg, receiver, return_addr ) , )*


			_ => panic!( "got wrong service: {:?}", x ),
		}
	}
}



/// Concrete type for creating recipients for remote Services in this thespis::ServiceMap.
//
#[ derive( Clone, Debug ) ]
//
pub struct ServicesRecipient
{
	peer: Addr<$peer_type>
}


impl ServicesRecipient
{
	pub fn new( peer: Addr<$peer_type> ) -> Self
	{
		Self { peer }
	}


	async fn send_gen<S>( &mut self, msg: S ) -> ThesRes<()>

		where  S                    : Service<UniqueID=ServiceID> ,
				<S as Message>::Result: Serialize + DeserializeOwned,

	{
		let sid        = S::uid( stringify!( $module ).as_bytes() );
		let cid        = ConnID::null();
		let serialized = serde_cbor::to_vec( &msg )?;

		let mul        = MultiServiceImpl::new( sid, cid, Codecs::CBOR, serialized.into() );

		await!( self.peer.send( mul ) )
	}


	async fn call_gen<S>( &mut self, msg: S ) -> ThesRes<<S as Message>::Result>

		where  S                    : Service<UniqueID=ServiceID> ,
				<S as Message>::Result: Serialize + DeserializeOwned,

	{
		let sid        = S::uid( stringify!( $module ).as_bytes() );
		let cid        = ConnID::default();
		let serialized = serde_cbor::to_vec( &msg )?.into();

		let mul        = MultiServiceImpl::new( sid, cid, Codecs::CBOR, serialized );

		let call = Call::new( mul );
		let re   = await!( await!( self.peer.call( call ) )?? )?;

		Ok( des( &re.mesg() )? )
	}
}




impl<S> Recipient<S> for ServicesRecipient

	where  S                    : MarkServices + Service<UniqueID=ServiceID> + Serialize + DeserializeOwned ,
	      <S as Message>::Result: Serialize + DeserializeOwned                                              ,

{
	/// Send any thespis::Service message that has a impl Message<Result=()> to a remote actor.
	//
	fn send( &mut self, msg: S ) -> Response< ThesRes<()> >
	{
		self.send_gen( msg ).boxed()
	}


	/// Call a remote actor.
	/// TODO: does this return type have must use or should we specify that ourselves. Maybe on the
	///       type alias...
	//
	fn call( &mut self, msg: S ) -> Response< ThesRes<<S as Message>::Result> >
	{
		self.call_gen( msg ).boxed()
	}


	/// Obtain a clone of this recipient as a trait object.
	//
	fn clone_box( &self ) -> Box< dyn Recipient<S> >
	{
		box Self { peer: self.peer.clone() }
	}
}

}

}} // End of macro
