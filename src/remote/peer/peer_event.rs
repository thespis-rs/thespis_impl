use crate::{ import::*, remote::* };

#[ derive( Debug, Clone, PartialEq ) ]
//
pub enum PeerEvent
{
	ConnectionClosed                   ,
	ConnectionClosedByRemote           ,
	ConnectionError( ConnectionError ) ,
}
