use crate::import::*;



/// Information about an actor, id, name and type_name. It implements [`Identify`].
/// Can also generate a span for tracing.
//
#[ derive( Debug, Clone, PartialEq, Eq, Hash ) ]
//
pub struct ActorInfo
{
	pub(crate) id       : usize    ,
	pub(crate) name     : Arc<str> ,
	pub(crate) type_name: String   ,
}


impl ActorInfo
{
	/// Setup actor information.
	//
	pub(crate) fn new<A: Actor>( id: usize, name: Arc<str> ) -> Self
	{
		Self
		{
			id                               ,
			name                             ,
			type_name: Self::gen_name::<A>() ,
		}
	}


	/// Generate typename from generic.
	//
	fn gen_name<A: Actor>() -> String
	{
		let name = std::any::type_name::<A>();

		match name.split( "::" ).last()
		{
			Some(t) => t.to_string(),
			None    => name.to_string(),
		}
	}


	/// The type of the actor.
	//
	pub fn type_name( &self ) -> &str
	{
		&self.type_name
	}


	/// Obtain a [`tracing::Span`] identifying the actor with it's id and it's name if it has one.
	//
	pub fn span( &self ) -> Span
	{
		if self.name.is_empty()
		{
			error_span!( "actor", id = self.id, "type" = self.type_name() )
		}

		else
		{
			error_span!( "actor", id = self.id, "type" = self.type_name(), a_name = self.name.as_ref() )
		}
	}
}



impl Identify for ActorInfo
{
	fn id( &self ) -> usize
	{
		self.id
	}

	fn name( &self ) -> Arc<str>
	{
		self.name.clone()
	}
}



impl From<ActorInfo> for String
{
	fn from( info: ActorInfo ) -> String
	{
		format!( "{}", info )
	}
}


impl fmt::Display for ActorInfo 
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		if self.name.is_empty()
		{
			write!( f, "{}: id={}", self.type_name(), self.id )
		}

		else
		{
			write!( f, "{}: id={}, name={}", self.type_name(), self.id, self.name )
		}
	}
}
