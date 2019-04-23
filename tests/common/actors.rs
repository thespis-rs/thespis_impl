use super::import::*;

#[ cfg( feature = "remote" ) ]
use
{
	serde:: { Serialize, Deserialize },
};



#[ derive( Actor ) ] pub struct Sum( pub u64 );

#[cfg_attr(feature = "remote", derive(Serialize, Deserialize))] #[ derive( Debug ) ] pub struct Add( pub u64 );
#[cfg_attr(feature = "remote", derive(Serialize, Deserialize))] #[ derive( Debug ) ] pub struct Show;

impl Message for Add  { type Result = ();  }
impl Message for Show { type Result = u64; }



impl Handler< Add > for Sum
{
	fn handle( &mut self, msg: Add ) -> Return<()> { async move
	{
		trace!( "called sum with: {:?}", msg );

		self.0 += msg.0;

	}.boxed() }
}



impl Handler< Show > for Sum
{
	fn handle( &mut self, _msg: Show ) -> Return<u64> { async move
	{
		trace!( "called sum with: Show" );

		self.0

	}.boxed() }
}
