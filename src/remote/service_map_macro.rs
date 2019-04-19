use crate::{ * };


#[ macro_export ]
//
macro_rules! service_map
{

(
	module       : $module    : ident          ;
	peer_type    : $peer_type : ty             ;
	multi_service: $ms_type   : ty             ;
	send_and_call: $($sendable: ident),* $(,)? ;
	call_only    : $($callable: ident),* $(,)? ;

) =>

{

pub mod $module
{

use
{
	super          :: { * },
	::futures      :: { future::FutureExt },
	::thespis      :: { * },
	::thespis_impl :: { *, remote::*, single_thread::*, runtime::rt },
	::serde_cbor   :: { self, from_slice as des },
	::serde        :: { Serialize, Deserialize, de::DeserializeOwned },
	::std          :: { any::Any                                     },
};

// Mark the services part of this particular service map, so we can write generic impls only for them.
//
pub trait MarkServices {}

$(	impl MarkServices for $sendable {} )*
$(	impl MarkServices for $callable {} )*


#[ derive( Clone ) ]
//
pub struct Services;

impl Services
{
	pub fn recipient<S>( peer: Addr<$peer_type> ) -> impl Recipient<S>

	where  S                    : MarkServices + Service<UniqueID=ServiceID> + Serialize + DeserializeOwned ,
	      <S as Message>::Result: Serialize + DeserializeOwned                                              ,
	{
		ServicesRecipient::new( peer )
	}


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
	fn send_service( &self, msg: $ms_type, receiver: &Box< dyn Any > )
	{
		let x = msg.service().expect( "get service" );

		match x
		{
			$( _ if x == <$sendable>::uid( stringify!( $module ).as_bytes() ) =>

				Self::send_service_gen::<$sendable>( msg, receiver ) , )*

			_ => panic!( "got wrong service: {:?}", x ),
		}
	}


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

			$( _ if x == <$sendable>::uid( stringify!( $module ).as_bytes() ) =>

				Self::call_service_gen::<$sendable>( msg, receiver, return_addr ) , )*


			$( _ if x == <$callable>::uid( stringify!( $module ).as_bytes() ) =>

				Self::call_service_gen::<$callable>( msg, receiver, return_addr ) , )*


			_ => panic!( "got wrong service: {:?}", x ),
		}
	}
}



#[ derive( Clone ) ]
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
		let re   = await!( await!( self.peer.call( call ) )? )?;

		Ok( des( &re.mesg() )? )
	}
}




impl<S> Recipient<S> for ServicesRecipient

	where  S                    : MarkServices + Service<UniqueID=ServiceID> + Serialize + DeserializeOwned ,
	      <S as Message>::Result: Serialize + DeserializeOwned                                              ,

{
	fn send( &mut self, msg: S ) -> Response< ThesRes<()> >
	{
		self.send_gen( msg ).boxed()
	}


	fn call( &mut self, msg: S ) -> Response< ThesRes<<S as Message>::Result> >
	{
		self.call_gen( msg ).boxed()
	}


	fn clone_box( &self ) -> Box< dyn Recipient<S> >
	{
		box Self { peer: self.peer.clone() }
	}
}

}

}} // End of macro
