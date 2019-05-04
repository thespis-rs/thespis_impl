use crate :: { import::* };

/// All errors that can happen when receiving messages over the wire
/// These will be broadcast to observers, so you can act upon them if necessary.
//
#[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
//
pub enum ConnectionError
{
	DeserializationFailure        ,
	LostRelayBeforeCall           , // the remote should know which service thanks to connID
	LostRelayBeforeResponse       , // the remote should know which service thanks to connID
	UnkownService(Vec<u8>)        ,
	LostRelayBeforeSend(Vec<u8>)  , // here there is no ConnID, so we put the sid back in there.
}
