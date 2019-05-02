use crate :: { import::* };

#[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
//
pub enum ConnectionError
{
	DeserializationFailure  ,
	LostRelayBeforeCall     ,
	LostRelayBeforeSend     ,
	LostRelayBeforeResponse ,
	UnkownService(Vec<u8>)  ,
}
