use crate::{ import::* };

/// The main error type for thespis_impl. Use [`ThesRemoteError::kind()`] to know which kind of
/// error happened. This implements Eq, so you can use:
///
/// ```ignore
/// match return_a_result()
/// {
///    Err(e) =>
///    {
///       match e.kind()
///       {
///          ThesRemoteErrKind::MailboxFull{..} => println!( "{}", e ),
///          _ => {},
///       }
///
///       if let ThesRemoteErrKind::MailboxFull{..} = e.kind()
///       {
///          println!( "{}", e );
///       }
///    },
///
///    Ok(_) => {}
/// }
/// ```
//
#[ derive( Debug ) ]
//
pub struct ThesRemoteErr
{
	inner: FailContext<ThesRemoteErrKind>,
}



/// The different kind of errors that can happen when you use the thespis_impl API.
//
#[ derive( Clone, Eq, Debug, Fail ) ]
//
pub enum ThesRemoteErrKind
{
	#[ fail( display = "Cannot use peer after the connection is closed, operation: {}", operation ) ]
	//
	ConnectionClosed { operation: String },

	#[ fail( display = "Failed to deserialize: {}", what ) ]
	//
	Deserialize { what: String },

	#[ fail( display = "Tokio IO Error" ) ]
	//
	TokioIoError,

	#[ fail( display = "Failed to spawn a future in: {}", context ) ]
	//
	Spawn { context: String },
}



impl Fail for ThesRemoteErr
{
	fn cause( &self ) -> Option< &Fail >
	{
		self.inner.cause()
	}

	fn backtrace( &self ) -> Option< &Backtrace >
	{
		self.inner.backtrace()
	}
}

impl fmt::Display for ThesRemoteErr
{
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result
	{
		fmt::Display::fmt( &self.inner, f )
	}
}


impl ThesRemoteErr
{
	/// Allows matching on the error kind
	//
	pub fn kind( &self ) -> ThesRemoteErrKind
	{
		*self.inner.get_context()
	}
}

impl From<ThesRemoteErrKind> for ThesRemoteErr
{
	fn from( kind: ThesRemoteErrKind ) -> ThesRemoteErr
	{
		ThesRemoteErr { inner: FailContext::new( kind ) }
	}
}

impl From< FailContext<ThesRemoteErrKind> > for ThesRemoteErr
{
	fn from( inner: FailContext<ThesRemoteErrKind> ) -> ThesRemoteErr
	{
		ThesRemoteErr { inner: inner }
	}
}


impl PartialEq for ThesRemoteErrKind
{
	fn eq( &self, other: &Self ) -> bool
	{
		core::mem::discriminant( self ) == core::mem::discriminant( other )
	}
}


impl std::error::Error for ThesRemoteErr {}



