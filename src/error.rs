use crate::{ import::* };


#[ derive( Debug, Fail ) ]
//
pub enum ThesError
{
	#[ fail( display = "Cannot initialize executor twice" ) ]
	//
	DoubleExecutorInit,
}

