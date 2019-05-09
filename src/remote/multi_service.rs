mod service_id;
mod conn_id;
mod codecs;

pub use
{
	codecs     :: * ,
	service_id :: * ,
	conn_id    :: * ,
};



#[ cfg( feature = "tokio" ) ] pub mod tokio_codec   ;
#[ cfg( feature = "tokio" ) ] pub use tokio_codec::*;

use { crate :: { import::*, remote::error::* } };

const HEADER_LEN: usize = 36;

/// A multi service message.
///
/// The message format is as follows:
///
/// The format is little endian.
///
/// length  : the length in bytes of the payload
/// uid     : user chosen uid for the service
/// encoding: serialization codec of the request message
/// connID  : in case of a call, which requires a response, a unique random number
///           in case of a send, which does not require response, zero
/// message : the request message serialized with the specified codec
///
/// ```text
/// u64 length + payload ---------------------------------------------------------------|
///              16 bytes uid | 16 bytes connID | 4 bytes encoding | serialized message |
///              u128         | u128            | u32              | variable           |
/// -------------------------------------------------------------------------------------
/// ```
///
/// As soon as a codec determines from the length field that the entire message is read,
/// they can create a Multiservice from the bytes. In general creating a Multiservice
/// object should not perform a copy of the serialized message. It just provides a window
/// to look into the multiservice message to figure out if:
/// - the service uid is meant to be delivered to an actor on the current process or is to be relayed.
/// - if unknown, if there is a non-zero connID, in which case an error is returned to the sender
/// - if it is meant for the current process, deserialize it with `encoding` and send it to the correct actor.
///
/// The algorithm for receiving messages should interprete the message like this:
///
/// ```text
/// - msg for a local actor -> service uid is in our in_process table
///   - send                -> no connID
///   - call                -> connID
///
/// - msg for a relayed actor -> service uid is in our routing table
///   - send                  -> no connID
///   - call                  -> connID
///
/// - a send/call for an actor we don't know -> uid unknown: respond with error
///
/// - a response to a call    -> service uid zero, valid connID
///   - when the call was made, we gave a onshot-channel receiver to the caller,
///     look it up in our open connections table and put the response in there.
///     Maybe need 2 different tables, one for in process and one for rely.
///
/// - an error message (eg. deserialization failed on the remote) -> service uid and connID zero.
/// 	TODO: This is a problem. We should know which connection erred, so connID should be valid...
///
///
///   -> log the error, we currently don't have a way to pass an error to the caller.
///      We should provide a mechanism for the application to handle the errors.
///      The deserialized message shoudl be a string error.
///
/// Possible errors:
/// - Destination unknown (since an address is like a capability,
///   we don't distinguish between not permitted and unknown)
/// - Fail to deserialize message
/// ```
/// TODO: Do we really need sync here?
//
#[ derive( Debug, Clone, PartialEq, Eq ) ]
//
pub struct MultiServiceImpl<SID, CID, Codec>

	where Codec: CodecAlg + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
	      CID  : UniqueID + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
	      SID  : UniqueID + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
         Self : From< Bytes >                                                 ,
{
	bytes: Bytes,

	p1: PhantomData< Codec >,
	p2: PhantomData< CID   >,
	p3: PhantomData< SID   >,
}


impl<SID: 'static, CID: 'static, Codec: 'static> Message for MultiServiceImpl<SID, CID, Codec>

	where Codec: CodecAlg + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
	      CID  : UniqueID + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
	      SID  : UniqueID + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
         Self : From< Bytes >                                                 ,

{
	type Return = ();
}



impl<SID, CID, Codec> MultiServiceImpl<SID, CID, Codec>

	where Codec: CodecAlg + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
	      CID  : UniqueID + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
	      SID  : UniqueID + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
         Self : From< Bytes >                                                 ,

{

}


impl<SID, CID, Codec> MultiService for MultiServiceImpl<SID, CID, Codec>

	where Codec: CodecAlg + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
	      CID  : UniqueID + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
	      SID  : UniqueID + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
         Self : From< Bytes >                                                 ,
{
	type ServiceID = SID           ;
	type ConnID    = CID           ;
	type CodecAlg  = Codec         ;
	type Error     = ThesRemoteErr ;


	/// Beware: This can panic because of Buf.put
	//
	fn create( service: SID, conn_id: CID, encoding: Codec, mesg: Bytes ) -> Self
	{
		let mut bytes = BytesMut::with_capacity( HEADER_LEN + mesg.len() );

		bytes.put( service .into() );
		bytes.put( conn_id .into() );
		bytes.put( encoding.into() );
		bytes.put( mesg            );

		Self { bytes: bytes.into(), p1: PhantomData, p2: PhantomData, p3: PhantomData }
	}


	fn service ( &self ) -> Result< Self::ServiceID, ThesRemoteErr >
	{
		SID::try_from( self.bytes.slice(0 , 16) )
	}


	fn encoding( &self ) -> Result< Self::CodecAlg, ThesRemoteErr >
	{
		Codec::try_from( self.bytes.slice(32, HEADER_LEN) )
	}


	fn conn_id( &self ) -> Result< Self::ConnID, ThesRemoteErr >
	{
		CID::try_from( self.bytes.slice(16, 32) )
	}


	fn mesg( &self ) -> Bytes
	{
		self.bytes.slice_from( HEADER_LEN )
	}

	fn len( &self ) -> usize
	{
		self.bytes.len()
	}
}



impl<SID, CID, Codec> Into< Bytes > for MultiServiceImpl<SID, CID, Codec>

	where Codec: CodecAlg + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
	      CID  : UniqueID + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
	      SID  : UniqueID + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
         Self : From< Bytes >                                                 ,

{
	fn into( self ) -> Bytes
	{
		self.bytes
	}
}



impl<SID, CID, Codec> From< Bytes > for MultiServiceImpl<SID, CID, Codec>

	where Codec: CodecAlg + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
	      CID  : UniqueID + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
	      SID  : UniqueID + TryFrom< Bytes, Error=ThesRemoteErr > + Send + Sync,
         Self : From< Bytes >                                                 ,

{
	fn from( bytes: Bytes ) -> Self
	{
		Self { bytes, p1: PhantomData, p2: PhantomData, p3: PhantomData }
	}
}




