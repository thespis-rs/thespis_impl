use crate::{ import::* };


#[ derive( Debug, Fail ) ]
//
pub enum ThesError
{
	#[ fail( display = "Cannot initialize executor twice" ) ]
	//
	DoubleExecutorInit,

	#[ fail( display = "Not enough bytes for the stream to contain a full message" ) ]
	//
	NeedMoreBytes,
}

