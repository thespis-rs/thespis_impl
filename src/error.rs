use crate::{ import::* };


/// The main error type for thespis_impl. Use [`ThesError::kind()`] to know which kind of
/// error happened. This implements Eq, so you can use:
///
/// ```ignore
/// match return_a_result()
/// {
///    Err(e) =>
///    {
///       match e.kind()
///       {
///          ThesErrKind::MailboxFull{..} => println!( "{}", e ),
///          _ => {},
///       }
///
///       if let ThesErrKind::MailboxFull{..} = e.kind()
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
pub struct ThesErr
{
	inner: FailContext<ThesErrKind>,
}



/// The different kind of errors that can happen when you use the thespis_impl API.
//
#[ derive( Clone, Eq, Debug, Fail ) ]
//
pub enum ThesErrKind
{
	#[ fail( display = "Failed to spawn a future in: {}", context ) ]
	//
	Spawn { context: String },

	#[ fail( display = "Mailbox Full for: {}", actor ) ]
	//
	MailboxFull { actor: String },

	#[ fail( display = "Mailbox crashed before we could send the message, actor: {}", actor ) ]
	//
	MailboxClosed { actor: String },

	#[ fail( display = "Mailbox crashed after the message was sent, actor: {}", actor ) ]
	//
	MailboxClosedBeforeResponse { actor: String },

	#[ fail( display = "Cannot initialize global executor twice" ) ]
	//
	DoubleExecutorInit,
}



impl Fail for ThesErr
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

impl fmt::Display for ThesErr
{
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result
	{
		fmt::Display::fmt( &self.inner, f )
	}
}


impl ThesErr
{
	/// Allows matching on the error kind
	//
	pub fn kind( &self ) -> &ThesErrKind
	{
		self.inner.get_context()
	}
}

impl From<ThesErrKind> for ThesErr
{
	fn from( kind: ThesErrKind ) -> ThesErr
	{
		ThesErr { inner: FailContext::new( kind ) }
	}
}

impl From< FailContext<ThesErrKind> > for ThesErr
{
	fn from( inner: FailContext<ThesErrKind> ) -> ThesErr
	{
		ThesErr { inner: inner }
	}
}


impl PartialEq for ThesErrKind
{
	fn eq( &self, other: &Self ) -> bool
	{
		core::mem::discriminant( self ) == core::mem::discriminant( other )
	}
}

impl std::error::Error for ThesErr {}


