//2-Use AsyncWrite to treat output as a Future
//write our stream of whatever type as long as it is serializable to any asyncwriter, one item at a time in its JSON form.
use futures::{future::Future, Async, try_ready};
use tokio::prelude::*;
use failure::Error;
use serde::Serialize; //to convert to JSON

pub struct JWrite<S,W>{
    in_s:S, //in stream
    out_w:W, //out writer
    buff:Option<Vec<u8>>,
}

impl<S,W> JWrite<S,W> {
    pub fn new(in_s:S,out_w:W)->Self{
        JWrite{
            in_s,
            out_w,
            buff:None,
        }
    }
}

impl <S,I,W> Future for JWrite<S,W> where S:Stream<Item = I,Error=()>,I:Serialize, W:AsyncWrite{
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Result<Async<Self::Item>,Self::Error>{

        loop{
            match self.buff{
                None => { //if the buffer is empty, then we should fill it
                    let i = match try_ready!{self.in_s.poll()}{
                        Some(v) => v,
                        None => return Ok(Async::Ready(())), //all senders are dropped, no input
                    };
                    self.buff = serde_json::to_string(&i)
                        .map(|v|v.as_bytes().to_vec())
                        .ok();
                }
                Some(ref mut v) => {
                    let n = try_ready!{self.out_w.poll_write(v).map_err(|e| println!("couldnot write out"))};
                    if n == v.len(){
                        self.buff = None;
                    }else{
                        self.buff = Some(v.split_off(n));
                    }
                }
            }
        }
        Ok(Async::Ready(()))
    }
}

