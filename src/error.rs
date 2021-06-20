use crate::{ import::*, ActorInfo, DynError };

/// Result which has a ThesErr as error type.
//
pub type ThesRes<T> = Result<T, ThesErr>;

/// Errors that can happen in thespis_impl.
//
#[ derive( Debug ) ]
//
pub enum ThesErr
{
	/// Either the actor panicked during processing of the message or the mailbox was dropped.
	/// Only returned when doing a `call`.
	//
	ActorStoppedBeforeResponse
	{
		/// Actor information.
		//
		info: Arc<ActorInfo>,
	},


	/// You try to use a mailbox that is already closed. The mailbox can be closed by dropping all
	/// strong addresses to it or by dropping the future that is running it.
	///
	/// When you get this error, the mailbox is gone and the address should be dropped. It will never
	/// accept messages again.
	//
	MailboxClosed
	{
		/// Actor information.
		//
		info: Arc<ActorInfo>,

		/// Source error.
		//
		src : Option<DynError>,
	},


	// /// The mailbox cannot take more messages right now. This only happens on
	// /// try_send on a future channel. For the moment there is no try_send in thespis.
	// //
	// #[ error( "The mailbox cannot take more messages right now. For actor: {actor}" ) ]
	// //
	// MailboxFull
	// {
	// 	/// The actor concerned by the error.
	// 	//
	// 	actor: ActorInfo
	// },


	/// Failed to spawn the mailbox.
	//
	Spawn
	{
		/// Actor information.
		//
		info: Arc<ActorInfo>,

		/// Source error.
		//
		src : SpawnError,
	}
}


impl ThesErr
{
	/// Get to the actor information of the error.
	//
	pub fn actor_info( &self ) -> &ActorInfo
	{
		match self
		{
			Self::ActorStoppedBeforeResponse { info, .. } => info,
			Self::MailboxClosed              { info, .. } => info,
			Self::Spawn                      { info, .. } => info,
		}
	}
}


impl Error for ThesErr
{
	fn source( &self ) -> Option< &(dyn Error + 'static) >
	{
		match &self
		{
			Self::ActorStoppedBeforeResponse {      .. } => None ,
			Self::MailboxClosed              { src, .. } => src.as_ref().map(|e| {let a: &(dyn Error + 'static) = e.as_ref(); a}  ) ,
			Self::Spawn                      { src, .. } => Some(src) ,
		}
	}
}


impl fmt::Display for ThesErr
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		match &self
		{
			ThesErr::ActorStoppedBeforeResponse{ info, .. } =>

				write!( f, "The mailbox was closed before the result of the computation got returned upon `call`. For actor: {}", info ),

			ThesErr::MailboxClosed{ info, .. } =>

				write!( f, "You try to use a mailbox that is already closed. For actor: {}", info ),

			ThesErr::Spawn{ info, .. } =>

				write!( f, "Failed to spawn the mailbox for actor: {}", info ),
		}
	}
}


impl PartialEq for ThesErr
{
	fn eq( &self, other: &Self ) -> bool
	{
		   std::mem::discriminant( self ) == std::mem::discriminant( other )
		&& self.actor_info()              == other.actor_info()
	}
}

impl Eq for ThesErr {}

