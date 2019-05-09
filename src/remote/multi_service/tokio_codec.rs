use crate::{ import::*, remote::error::* };


impl From<tokio::io::Error> for ThesRemoteErr
{
	fn from( e: tokio::io::Error ) -> Self
	{
		ThesRemoteErrKind::TokioIoError.into()
	}
}



// TODO: - max_length!
//
pub struct MulServTokioCodec<MS> where MS: MultiService
{
	pub _p: PhantomData<MS>,
}


impl<MS> MulServTokioCodec<MS> where MS: MultiService
{
	pub fn new() -> Self
	{
		Self{ _p: PhantomData }
	}
}


impl<MS> Decoder for MulServTokioCodec<MS>

	where MS: MultiService,
{
	type Item  = MS            ;
	type Error = ThesRemoteErr ;

	fn decode( &mut self, buf: &mut BytesMut ) -> Result< Option<Self::Item>, Self::Error >
	{
		// Minimum length of an empty message is the u64 indicating the length
		//
		if buf.len() < 8  { return Ok( None ) }


		// parse the first 8 bytes to find out the total length of the message
		//
		let mut len = buf[..8]

			.as_ref()
			.read_u64::<LittleEndian>()
			.context( ThesRemoteErrKind::Deserialize{ what: "Tokio codec: Length".to_string() } )?

			as usize
		;


		if buf.len() < len { return Ok( None ) }

		// We have an entire message.
		// Remove the length header, we won't need it anymore
		//
		buf.advance( 8 );
		len -= 8;

		// Consume the message
		//
		let buf = buf.split_to( len );

		// Convert
		//
		Ok( Some( MS::from( Bytes::from( buf ) ) ) )
	}
}


// TODO: zero copy encoding. Currently we copy bytes in the output buffer.
// In principle we would like to not have to serialize the inner message
// before having access to this buffer.
//
impl<MS> Encoder for MulServTokioCodec<MS>

	where MS: MultiService
{
	type Item  = MS            ;
	type Error = ThesRemoteErr ;

	fn encode( &mut self, item: Self::Item, buf: &mut BytesMut ) -> Result<(), Self::Error>
	{
		let len = item.len() + 8;
		buf.reserve( len );

		let mut wtr = vec![];
		wtr.write_u64::<LittleEndian>( len as u64 ).expect( "Tokio codec encode: Write u64 to vec" );

		buf.put( wtr         );
		buf.put( item.into() );

		Ok(())
	}
}


#[ cfg(test) ]
//
mod tests
{
	// Tests:
	//
	// 1. A valid MultiServiceImpl, encoded + decoded should be identical to original.
	// 2. Send like 2 and a half full objects, test that 2 correctly come out, and there is the
	//    exact amount of bytes left in the buffer for the other half.
	//
	use crate::{ *, remote::* };
	use super::{ *, assert_eq };


	type MulServ = MultiServiceImpl<ServiceID, ConnID, Codecs>;

	fn ashex( buf: &[u8] ) -> String
	{
		let mut f = String::new();

		for byte in buf
		{
			std::fmt::write( &mut f, format_args!( "{:02x}", byte ) ).expect( "Create hex string from slice" )
		}

		f
	}


	fn empty_data() -> MulServ
	{
		let mut buf = BytesMut::with_capacity( 1 );
		buf.put( 0u8 );

		let m = MultiServiceImpl::create( ServiceID::from_seed( b"Empty Message" ), ConnID::default(), Codecs::CBOR, buf.into() );

		m
	}

	fn full_data() -> MulServ
	{
		let mut buf = BytesMut::with_capacity( 5 );
		buf.put( b"hello".to_vec() );

		MultiServiceImpl::create( ServiceID::from_seed( b"Full Message" ), ConnID::default(), Codecs::CBOR, buf.into() )
	}


	#[test]
	//
	fn empty()
	{
		let mut codec: MulServTokioCodec<MulServ> = MulServTokioCodec::new();

		let mut buf   = BytesMut::new();
		let     data  = empty_data();
		let     data2 = data.clone();

		codec.encode( data, &mut buf ).expect( "Encoding empty" );

		assert_eq!
		(
			data2,

			codec.decode( &mut buf )

				.expect( "No errors should occur"                      )
				.expect( "There should be some data (eg. Not Ok(None)" )
		);
	}


	#[test]
	//
	fn full()
	{
		let mut codec: MulServTokioCodec<MulServ> = MulServTokioCodec::new();

		let mut buf   = BytesMut::new();
		let     data  = full_data();
		let     data2 = data.clone();

		codec.encode( data, &mut buf ).expect( "Encoding empty" );

		assert_eq!
		(
			data2,

			codec.decode( &mut buf )

				.expect( "No errors should occur"                      )
				.expect( "There should be some data (eg. Not Ok(None)" )
		);
	}


	#[test]
	//
	fn partials()
	{
		let mut codec: MulServTokioCodec<MulServ> = MulServTokioCodec ::new();

		let mut buf = BytesMut::new();

		let     empty  = empty_data();
		let     full   = full_data ();

		let     empty2 = empty.clone();
		let     full2  = full .clone();
		let     full3  = full .clone();

		codec.encode( empty, &mut buf ).expect( "Encoding empty" ); // 45 bytes
		codec.encode( full , &mut buf ).expect( "Encoding full"  ); // 49 bytes
		codec.encode( full2, &mut buf ).expect( "Encoding full"  ); // 49 bytes

		// total is 143
		// remove last byte
		//
		buf.truncate( 142 );

		assert_eq!( empty2, codec.decode( &mut buf ).expect( "Decode empty" ).expect( "Not None" ) );
		assert_eq!(  full3, codec.decode( &mut buf ).expect( "Decode empty" ).expect( "Not None" ) );
		assert_eq!( 48, buf.len() ); // there should be exactly 48 bytes sitting there waiting for the last.
	}
}
