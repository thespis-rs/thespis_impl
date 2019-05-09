use crate::*;

mod a
{
	use crate::*;

	service_map!
	(
		namespace:     remotes   ;
		peer_type:     MyPeer    ;
		multi_service: MS        ;
		services     : Add, Show ;
	);


	service_map!
	(
		namespace:     others  ;
		peer_type:     MyPeer  ;
		multi_service: MS      ;
		services     : Add     ;
	);
}

mod b
{
	use crate::*;

	service_map!
	(
		namespace:     remotes ;
		peer_type:     MyPeer  ;
		multi_service: MS      ;
		services     : Add     ;
	);
}


// Verify that the same service, in a different namespace has different service id.
//
#[ test ]
//
fn sid_diff_for_diff_ns()
{
	assert_ne!( <Add as Service<a::remotes::Services>>::sid(), <Add as Service<a::others::Services>>::sid() );
}


// Verify that the same service, in a different namespace has different service id.
//
#[ test ]
//
fn sid_diff_for_diff_service()
{
	assert_ne!( <Add as Service<a::remotes::Services>>::sid(), <Show as Service<a::remotes::Services>>::sid() );
}


// Verify that the same service in different service maps with the same namespace has identical sid
//
#[ test ]
//
fn sid_same_for_same_ns()
{
	assert_eq!( <Add as Service<a::remotes::Services>>::sid(), <Add as Service<b::remotes::Services>>::sid() );
}
