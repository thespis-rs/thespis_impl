use super::import::*;


#[ allow( dead_code ) ]
//
#[ derive( Actor ) ] pub struct SumNoSend( pub u64 );

impl !Send for SumNoSend {}
impl !Sync for SumNoSend {}


#[ derive( Actor ) ] pub struct Sum( pub u64 );

#[ derive( Debug ) ] pub struct Add( pub u64 );
#[ derive( Debug ) ] pub struct Show;

impl Message for Add  { type Return = ();  }
impl Message for Show { type Return = u64; }



impl Handler< Add > for Sum
{
	fn handle( &mut self, msg: Add ) -> Return<()> { Box::pin( async move
	{
		trace!( "called sum with: {:?}", msg );

		self.0 += msg.0;

	})}
}



impl Handler< Show > for Sum
{
	fn handle( &mut self, _msg: Show ) -> Return<u64> { Box::pin( async move
	{
		trace!( "called sum with: Show" );

		self.0

	})}
}



impl Handler< Add > for SumNoSend
{
	fn handle( &mut self, _: Add ) -> Return<()> { Box::pin( async move
	{
		unreachable!( "Can't spawn !Send actor on threadpool" );

	})}

	fn handle_local( &mut self, msg: Add ) -> ReturnNoSend<()> { Box::pin( async move
	{
		trace!( "called sum with: {:?}", msg );

		self.0 += msg.0;

	})}
}



impl Handler< Show > for SumNoSend
{
	fn handle( &mut self, _: Show ) -> Return<u64> { Box::pin( async move
	{
		unreachable!( "Can't spawn !Send actor on threadpool" );

	})}

	fn handle_local( &mut self, _msg: Show ) -> ReturnNoSend<u64> { Box::pin( async move
	{
		trace!( "called sum with: Show" );

		self.0

	})}
}
