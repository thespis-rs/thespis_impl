use crate::{ import::*, remote::error::* };


/// A connection identifier. This allows to match incoming packets to requests sent earlier.
///
/// It is currently 128 bits, if created with the default impl here it will be random data.
///
/// TODO: keep it DRY, lot's of code duplication with ServiceID.
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
		Self::in_place( BytesMut::with_capacity( 16 ) )
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
	type Error = ThesRemoteErr;

	fn try_from( bytes: Bytes ) -> Result<Self, Self::Error>
	{
		Ok( Self { bytes } )
	}
}



impl fmt::Display for ConnID
{
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result
	{
		write!( f, "{:?}", self )
	}
}



impl fmt::Debug for ConnID
{
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result
	{
		for byte in &self.bytes
		{
			write!( f, "{:02x}", byte )?
		}

		Ok(())
	}
}


impl UniqueID for ConnID
{
	/// Seed the uniqueID. It might be data that will be hashed to generate the id.
	/// An identical input here should always give an identical UniqueID.
	///
	/// This is not common for ConnID, since usually on wants it to be random
	/// (as the impl Default provides), but since the trait UniqueID requires it,
	/// you can get a hashed version here. This uses xxhash.
	///
	/// Currently xxhash only generates 64bits of data, so the collsision strength of this
	/// is only 64bits.
	//
	fn from_seed( data: &[u8] ) -> Self
	{
		let     b: RandomXxHashBuilder = Default::default();
		let mut h                      = b.build_hasher();

		for byte in data
		{
			h.write_u8( *byte );
		}

		// TODO: this is broken right now. Only creates 8 bytes instead of 16.
		//       We need unit tests for this, and also  the in_place method is not
		//       really in_place,...
		//
		//       Ok, my bad, i fixed it before writing the unit test...
		//
		let mut wtr = vec![];
		wtr.write_u64::<LittleEndian>( h.finish() ).unwrap();
		wtr.write_u64::<LittleEndian>( 0          ).unwrap();

		Self { bytes: Bytes::from( wtr ) }
	}


	fn null() -> Self
	{
		// The format of the multiservice message requires this to be 128 bits, so add a zero
		// We will have 128bit hash here when xxhash supports 128bit output.
		//
		let mut wtr = vec![];
		wtr.write_u64::<LittleEndian>( 0 ).unwrap();
		wtr.write_u64::<LittleEndian>( 0 ).unwrap();

		Self { bytes: Bytes::from( wtr ) }
	}


	fn is_null( &self ) -> bool
	{
		self.bytes.iter().all( |b| *b == 0 )
	}
}


