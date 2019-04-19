#![ allow( unused_imports, dead_code ) ]
#![ feature( await_macro, async_await, futures_api, arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias, box_syntax, box_patterns, todo_macro, try_trait, optin_builtin_traits ) ]


mod common;




#[test]
//
fn remotes()
{
	rt::init( box TokioRT::default() ).expect( "We only set the executor once" );



	rt::spawn( listen  ).expect( "Spawn listen"  );
	rt::spawn( connect ).expect( "Spawn connect" );
	rt::run();
}
