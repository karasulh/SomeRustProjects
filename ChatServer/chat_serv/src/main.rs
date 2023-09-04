//TCPConnection creates an asyncRead and asyncWrite. AsyncRead is a poll method which we can use grab data. Alternatively we can future or stream.
//One of the problem of TCP Connection, you couldnot understand the person send everything that you want to use before you send anything back.

//to use tokio::spawn, we must be inside of future
//"wait" : blocks the current thread until the future result. We shouldnot use it inside a future because it will stop other futures from being run.
//Instead of "wait" use tokio::spawn inside future's closure and use "and_then" or "then" on current future.

use futures::sync::{oneshot, mpsc};
//3-Listening over TCP
use tokio::net::TcpListener;
use tokio::prelude::*;

use serde_derive::*;

use chatbox::{ChatBox,Request};

mod j_read;
mod j_write;

//4-Converting the input data we are getting into our Rust struct type using Serde 
//send the below messages on telnet
//11:{"since":0}
//35:{"mess":{"name":"mt","tx":"hello"}}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Message{
    name: String,
    tx: String,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ServerMessage{
    mess: Option<Message>,
    since: Option<usize>,
}


fn main() {
    let addr= "127.0.0.1:8088".parse().unwrap();
    let lis = TcpListener::bind(&addr).expect("couldnot bind address");
    
    //5-Completing the Application with Chatbox
    let (ch_box,ch_s) = ChatBox::new();

    let fut = lis.incoming().for_each(move |sock|{ //takes closure, returns future; all of these run, future is complete
        let ch_s = ch_s.clone();
        let (sock_r,sock_w) = sock.split();
        let (fin_s,fin_r) = mpsc::channel(10); //final read and send
        let write_f = j_write::JWrite::new(fin_r,sock_w);
        tokio::spawn(write_f);

        let rd = j_read::JRead::new(sock_r).for_each(move |s|{ //JRead is a stream; to turn it to future use for_each
            let v:ServerMessage = serde_json::from_str(&s)?;
            println!("received:{:?}",v);
            if let Some(m) = v.mess{
                let f = ch_s.clone()
                            .send(Request::Put(m))
                            .map(|_|())
                            .map_err(|_| println!("coudlnot send message to chatbox"));
                tokio::spawn(f);
            }

            if let Some(n) = v.since{
                let (os_s,os_r) = oneshot::channel();
                let fc = fin_s.clone();
                let f = ch_s.clone()
                            .send(Request::Since(n,os_s))
                            .map_err(|e|println!("couldnot send since to chatbox"))
                            .and_then(|_|os_r.map_err(|e|println!("couldnot get from the chatbox oneshot")))
                            .and_then(move |v| fc.send(v).map_err(|e|println!("couldnot send to fin_c")))
                            .map(|_|());
                tokio::spawn(f);
            }

            //println!("received:{}",s);
            Ok(())
        }).map_err(|_|());
        tokio::spawn(rd);
        Ok(())
    }).map_err(|e|println!("Listening Err: {:?}",e));

    tokio::run(fut.join(ch_box).map(|_|())); //ch_box joins to another future
    //tokio::run(fut);

    println!("Hello, world!");
}
