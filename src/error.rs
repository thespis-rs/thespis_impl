use crate::{ import::* };


#[ derive( Debug, Fail ) ]
//
pub enum ThesError
{
	#[ fail( display = "Cannot initialize executor twice" ) ]
	//
	DoubleExecutorInit,


	#[ fail( display = "Cannot send messages to a closed connection" ) ]
	//
	PeerSendAfterCloseConnection,

	// #[ fail( display = "Not enough bytes for the stream to contain a full message" ) ]
	// //
	// NeedMoreBytes,

	// /// Connection will be closed when this happens, because it might mean that the stream has become corrupt.
	// //
	// #[ fail( display = "Decoding a frame coming in over the network has failed. The connection has been closed." ) ]
	// //
	// CorruptFrame,
}

