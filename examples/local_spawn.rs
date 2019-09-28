#![feature(arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias)]

#![allow(dead_code, unused_imports)]


use
{
    futures::{future::{Future, FutureExt}, task::{LocalSpawn, SpawnExt, LocalSpawnExt}, executor::LocalPool},
    std::{pin::Pin},
    log::{*},
    thespis::{*},
    thespis_impl::{*},
    async_runtime::rt,
};


#[derive(Actor)]
//
struct MyActor;

struct Ping(String);


impl Message for Ping
{
    type Return = String;
}


impl Handler<Ping> for MyActor
{
    fn handle(&mut self, _msg: Ping) -> Return<String> {
        Box::pin(async move
            {
                "pong".into()
            })
    }
}


fn main()
{
    let actor = MyActor;
    let inbox = Inbox::new();
    let mut addr = inbox.addr();
    let future = inbox.start_fut_local(actor);

    let mut pool = futures::executor::LocalPool::new();
    pool.spawner().spawn_local(future).unwrap();


    pool.spawner().spawn_local(async move {
        let result = addr.call(Ping("ping".into())).await.expect("Call failed");
        assert_eq!("pong".to_string(), result);
        dbg!(result);
    }).unwrap();

    pool.run();
}
