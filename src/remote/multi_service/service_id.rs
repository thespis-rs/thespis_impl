use crate::{ import::* };


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
	type Error = Error;

	fn try_from( bytes: Bytes ) -> Result< Self, Error >
	{
		Ok( Self { bytes } )
	}
}


impl fmt::Display for ServiceID
{
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result
	{
		write!( f, "{:x?}", self )
	}
}



impl fmt::Debug for ServiceID
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


impl UniqueID for ServiceID
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

		// The format of the multiservice message requires this to be 128 bits, so add a zero
		// We will have 128bit hash here when xxhash supports 128bit output.
		//
		let mut wtr = vec![];
		wtr.write_u64::<LittleEndian>( h.finish() ).unwrap();
		wtr.write_u64::<LittleEndian>( 0          ).unwrap();

		Self { bytes: Bytes::from( wtr ) }
	}
}


