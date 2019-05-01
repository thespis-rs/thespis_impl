use crate::{ import::*, remote::* };

#[ derive( Debug, Clone, PartialEq ) ]
//
pub enum PeerEvent
{
	Closed                   ,
	ClosedByRemote           ,
	Error( ConnectionError ) ,
}
