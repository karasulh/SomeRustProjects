//Future is a trait from a library in the past, and now it is in the std library. It has "poll" method which return async item when it is ready.
//In this case, we use Future trait from library Futures
//Future concludes item and error type.
//Tokio and futures provides asynchronous running code that is they work at seperate times but they are not seperate threads.
//Future never blocks the threads and it returns either "ready" or "not ready".

//Future is a struct in our case.

use futures::{Future, Async, sync::mpsc};
use tokio::prelude::*;
use tokio_channel::oneshot; 
use std::fmt::Debug;

pub enum Request<M>{
    Put(M),
    Since(usize,oneshot::Sender<Vec<M>>)
}

pub struct ChatBox<M>{
    store: Vec<M>,
    ch_r: mpsc::Receiver<Request<M>>, //multi producer single consumer
}

impl<M> ChatBox<M>{
    pub fn new() -> (Self,mpsc::Sender<Request<M>>){
        println!("creating Chatbox");
        let (ch_s,ch_r) = mpsc::channel(10);
       (
            ChatBox { 
                store: Vec::new(), 
                ch_r 
            },
            ch_s
        )
    }
}

//When it is "NotReady", we shouldnot return "NotReady" because nothing is going to wake our thread up.
//Only ok to return it, when we have told something to wake our function or we receive NotReady from a function we call

impl<M:Debug+Clone> Future for ChatBox<M>{
    type Item = (); //String;
    type Error = ();
    fn poll(&mut self) -> Result<Async<Self::Item>,Self::Error> {

        loop{
            let rq = match {self.ch_r.poll()?} {
                Async::NotReady => return Ok(Async::NotReady),
                Async::Ready(Some(v)) => v,
                Async::Ready(None) =>  return Ok(Async::Ready(())),
            };
            match rq{
                Request::Put(m) => {
                    println!("got message {:?}",m);
                    self.store.push(m);
                }
                Request::Since(n,ch)=> {
                    println!("got request {:?}",n);
                    let res = if n >= self.store.len(){
                        Vec::new()
                    } else{
                        Vec::from(&self.store[n..])
                    };
                    ch.send(res).ok();
                }
            }
        }
        
        //return Ok(Async::Ready("hello".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::lazy; //If there is two tokio::run, one of them first starts and when it finishes, other one starts. To prevent this use lazy future.

    #[test] 
    fn it_works() {
        //******4 //Getting Data Back out of Futures in Synchronous Code
        let (ts_s,ts_r) = mpsc::channel(10);
        //******3 //One shot communication across simple channels
        let f = lazy(move||{
            let (f,ch_s) = ChatBox::new();
            tokio::spawn(f); //use spawn not run in lazy function to run in another thread //f is future
            for i in 0..5 {
                let tss = ts_s.clone();
                let ch2 = ch_s.clone();
                let (os_s,os_r) = oneshot::channel();
                let f2 = ch_s
                    .clone()
                    .send(Request::Put(i))
                    .and_then(|_| ch2.send(Request::Since(0, os_s)))
                    .map_err(|e|println!("{:?}",e))
                    .and_then(|_|os_r.map_err(|_|()))
                    //.map(move |res|println!("res {} = {:?}",i,res))
                    //.map_err(|e|println!("{:?}",e));
                    .and_then(move |res|{
                        println!("res {} = {:?}",i,res);
                       tss.send(res).map_err(move |_|println!("couldnot send {}",i)) 
                    })
                    .map(|_|());
                tokio::spawn(f2);

            }
            Ok(())
        });
        tokio::run(f);

        let mut longest = 0;
        for v in ts_r.wait(){
            longest = std::cmp::max(longest, v.unwrap().len());
        }
        assert_eq!(longest,5);

        //******2
        // let f = lazy(||{
        //     let (f,ch_s) = ChatBox::new();
        //     tokio::spawn(f); //use spawn not run in lazy function to run in another thread
        //     tokio::spawn(
        //         ch_s.send(Request::Put(3))
        //         .map(|_|())
        //         .map_err(|e|println!("Send Error: {:?}",e))
        //     );
        //     Ok(())
        // });
        // tokio::run(f);

        //*******1
        //let f = ChatBox::new();
        //let p = f.map(|s| println!("{}",s));
        //println!("Beginning");
        //tokio::run(p);
        //println!("Ending");
        //panic!(); //to see prints
    }
}
