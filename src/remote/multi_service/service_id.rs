use crate::{ import::*, remote::error::* };


/// A unique identifier for a service that is exposed to other processes. This will allow
/// identifying the type to which the payload needs to be deserialized and the actor to which
/// this message is to be delivered.
///
/// Ideally we want to use 128 bits here to have globally unique identifiers with little chance
/// of collision, but we use xxhash which for the moment only supports 64 bit.
//
#[ derive( Clone, PartialEq, Eq, Hash ) ]
//
pub struct ServiceID
{
	bytes: Bytes,
}



/// Internally is also represented as Bytes, so you just get a copy.
//
impl Into< Bytes > for ServiceID
{
	fn into( self ) -> Bytes
	{
		self.bytes
	}
}


/// The object will just keep the bytes as internal representation, no copies will be made
//
impl TryFrom< Bytes > for ServiceID
{
	type Error = ThesRemoteErr;

	fn try_from( bytes: Bytes ) -> Result<Self, ThesRemoteErr>
	{
		Ok( Self { bytes } )
	}
}


impl fmt::Display for ServiceID
{
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result
	{
		write!( f, "{:?}", self )
	}
}



impl fmt::Debug for ServiceID
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


impl UniqueID for ServiceID
{
	/// Seed the uniqueID. It might be data that will be hashed to generate the id.
	/// An identical input here should always give an identical UniqueID.
	//
	fn from_seed( data: &[u8] ) -> Self
	{
		let mut h = XxHash::default();

		for byte in data
		{
			h.write_u8( *byte );
		}

		// The format of the multiservice message requires this to be 128 bits, so add a zero
		// We will have 128bit hash here when xxhash supports 128bit output.
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


#[cfg(test)]
//
mod tests
{
	// What's tested:
	// 1. Identical input data should give identical hash.
	//
	use super::{ *, assert_eq };


	#[test]
	//
	fn identical()
	{
		let sid  = ServiceID::from_seed( b"hi from seed" );
		let sid2 = ServiceID::from_seed( b"hi from seed" );

		assert_eq!( sid, sid2 );
	}


	#[test]
	//
	fn debug()
	{
		let bytes: Bytes = vec![ 0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, ].into();
		let sid = ServiceID::try_from( bytes ).unwrap();

		assert_eq!( "000102030405060708090a0b0c0d0e0f", &format!( "{:?}", sid ) );
	}
}


