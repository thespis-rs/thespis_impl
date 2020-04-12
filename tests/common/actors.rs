use super::import::*;


#[ allow( dead_code ) ]
//
#[ derive( Actor ) ] pub struct SumNoSend( pub u64, PhantomData<*mut ()> );

impl SumNoSend
{
	#[ allow( dead_code ) ]
	//
	pub fn new( start: u64 ) -> SumNoSend
	{
		Self( start, PhantomData )
	}
}


#[ derive( Actor ) ] pub struct Sum( pub u64 );

#[ derive( Debug ) ] pub struct Add( pub u64 );
#[ derive( Debug ) ] pub struct Show;

impl Message for Add  { type Return = ();  }
impl Message for Show { type Return = u64; }



impl Handler< Add > for Sum
{
	#[async_fn] fn handle( &mut self, msg: Add )
	{
		trace!( "called sum with: {:?}", msg );

		self.0 += msg.0;
	}
}



impl Handler< Show > for Sum
{
	#[async_fn] fn handle( &mut self, _msg: Show ) -> u64
	{
		trace!( "called sum with: Show" );

		self.0
	}
}



impl Handler< Add > for SumNoSend
{
	#[async_fn] fn handle( &mut self, _: Add )
	{
		unreachable!( "Can't spawn !Send actor on threadpool" );
	}

	#[async_fn_nosend] fn handle_local( &mut self, msg: Add )
	{
		trace!( "called sum with: {:?}", msg );

		self.0 += msg.0;
	}
}



impl Handler< Show > for SumNoSend
{
	#[async_fn] fn handle( &mut self, _: Show ) -> u64
	{
		unreachable!( "Can't spawn !Send actor on threadpool" );
	}

	#[async_fn_nosend] fn handle_local( &mut self, _msg: Show ) -> u64
	{
		trace!( "called sum with: Show" );

		self.0
	}
}
