use crate::{ import::*, remote::error::* };


/// The supported encodings for this implementation. Values come from the
/// multiformat's multicodec table where possible.
/// https://github.com/multiformats/multicodec/blob/master/table.csv
//
#[ derive( Debug, Clone, PartialEq, Eq, Hash, FromPrimitive, ToPrimitive ) ]
//
pub enum Codecs
{
	CBOR = 0x51  ,
	UTF8 = 0x4000, // not in multicodecs for now
}

impl CodecAlg for Codecs { type Error = ThesRemoteErr; }


impl fmt::Display for Codecs
{
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result
	{
		match self
		{
			Codecs::CBOR => write!( f, "CBOR" )?,
			Codecs::UTF8 => write!( f, "UTF8" )?,
		}

		Ok(())
	}
}




impl Into< Bytes > for Codecs
{
	fn into( self ) -> Bytes
	{
		let mut wtr = vec![];

		wtr.write_u32::<LittleEndian>
		(
			self.to_u32().expect( "convert Codecs enum to u32" )
		)

			.expect( "write u32 to Vec<u8>" )
		;

		Bytes::from( wtr )
	}
}



impl TryFrom< Bytes > for Codecs
{
	type Error = ThesRemoteErr;

	fn try_from( bytes: Bytes ) -> Result<Self, Self::Error>
	{
		let mut rdr = Cursor::new( bytes.as_ref() );

		let num = rdr.read_u32::<LittleEndian>().expect( "Read Codec from Bytes" );

		Codecs::from_u32( num ).ok_or( ThesRemoteErrKind::Deserialization{ what: "Codec".to_string() }.into() )
	}
}



#[ cfg(test) ]
//
mod tests
{
	// Tests:
	//
	// 1. to bytes and back
	// 2. content of binary form is correct
	//
	use crate::{ *, remote::* };
	use super::{ *, assert_eq };



	fn ashex( buf: &[u8] ) -> String
	{
		let mut f = String::new();

		for byte in buf
		{
			std::fmt::write( &mut f, format_args!( "{:02x}", byte ) ).expect( "Create hex string from slice" )
		}

		f
	}


	#[test]
	//
	fn to_bytes()
	{
		let buf: Bytes = Codecs::CBOR.into();

		assert_eq!( 4           , buf.len()                                        );
		assert_eq!( "51000000"  , ashex( &buf )                                    );
		assert_eq!( Codecs::CBOR, Codecs::try_from( buf ).expect( "decode bytes" ) );
	}
}
