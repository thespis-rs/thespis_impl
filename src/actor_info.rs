use crate::import::*;



/// Information about an actor, id, name and type_name.
/// Can also generate a span for tracing.
//
#[ derive( Debug, Clone, PartialEq, Eq, Hash ) ]
//
pub struct ActorInfo
{
	pub(crate) id       : usize              ,
	pub(crate) name     : Option< Arc<str> > ,
	pub(crate) type_name: String             ,
}


impl ActorInfo
{
	/// Setup actor information.
	//
	pub(crate) fn new<A: Actor>( id: usize, name: Option<Arc<str>> ) -> Self
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
	pub fn gen_name<A: Actor>() -> String
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
		if let Some( name ) = &self.name
		{
			error_span!( "actor", id = self.id, "type" = self.type_name(), name = name.as_ref() )
		}

		else
		{
			error_span!( "actor", id = self.id, "type" = self.type_name() )
		}
	}
}



impl Identify for ActorInfo
{
	fn id( &self ) -> usize
	{
		self.id
	}

	fn name( &self ) -> Option< Arc<str> >
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
		if let Some(name) = &self.name
		{
			write!( f, "{}: id={}, name={}", self.type_name(), self.id, name )
		}

		else
		{
			write!( f, "{}: id={}", self.type_name(), self.id )
		}
	}
}
