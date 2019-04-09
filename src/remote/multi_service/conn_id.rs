use crate::{ import::* };


/// A connection identifier. This allows to match incoming packets to requests sent earlier.
///
/// It is currently 128 bits, if created with the default impl here it will be random data.
//
#[ derive( Clone, PartialEq, Eq, Hash ) ]
//
pub struct ConnID
{
	bytes: Bytes ,
}



impl ConnID
{
	/// Create a ConnID directly into the provided memory
	/// TODO: for the moment we copy from a Vec, write directly into buf.
	//
	pub fn in_place( mut buf: BytesMut ) -> Self
	{
		let mut rng = rand::thread_rng();

		// u128 doesn't work in wasm and serde is being a pain, so 2 u64
		//
		let a = rng.gen::<u64>();
		let b = rng.gen::<u64>();

		let mut wtr = vec![];
		wtr.write_u64::<LittleEndian>( a ).unwrap();
		wtr.write_u64::<LittleEndian>( b ).unwrap();

		buf.extend( wtr );

		Self { bytes: Bytes::from( buf ) }
	}
}



/// Generate a random ConnID
//
impl Default for ConnID
{
	fn default() -> Self
	{
		Self::in_place( BytesMut::with_capacity( 128 ) )
	}
}



impl Into< Bytes > for ConnID
{
	fn into( self ) -> Bytes
	{
		self.bytes
	}
}



impl TryFrom< Bytes > for ConnID
{
	type Error = Error;

	fn try_from( bytes: Bytes ) -> Result< Self, Error >
	{
		Ok( Self { bytes } )
	}
}



impl fmt::Display for ConnID
{
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result
	{
		write!( f, "{:x?}", self )
	}
}



impl fmt::Debug for ConnID
{
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result
	{
		for byte in &self.bytes
		{
			write!( f, "{:x}", byte )?
		}

		Ok(())
	}
}


impl UniqueID for ConnID
{
	/// Seed the uniqueID. It might be data that will be hashed to generate the id.
	/// An identical input here should always give an identical UniqueID.
	//
	fn from_seed( data: &[u8] ) -> Self
	{
		let     b: RandomXxHashBuilder = Default::default();
		let mut h                      = b.build_hasher();

		for byte in data
		{
			h.write_u8( *byte );
		}

		let mut wtr = vec![];
		wtr.write_u64::<LittleEndian>( h.finish() ).unwrap();

		Self { bytes: Bytes::from( wtr ) }
	}
}


