//1-Use AsyncRead to treat input as an Asynchronous Stream

use futures::try_ready;
use tokio::prelude::*;
use failure::Error;

enum Rding{
    Len(usize), //length of trying to read 
    Val(usize,Vec<u8>), //readling bytes into vec
}

pub struct JRead<R:AsyncRead>{
    buf: [u8;500],
    buflen: usize,
    ptr: usize,
    reading: Rding,
    r:R, //asyncRead
}

impl<R:AsyncRead> JRead<R>{
    pub fn new(r:R) -> Self{
        JRead{ 
            buf: [0;500],
            buflen: 0,
            ptr: 0,
            reading: Rding::Len(0),
            r, //asyncRead
        }
    }
}

impl <R:AsyncRead> Stream for JRead<R>{
    type Item = String;
    type Error = Error;
    fn poll(&mut self) -> Result<Async<Option<Self::Item>>,Self::Error>{

        loop{
            if self.ptr == self.buflen{ //We are at the end of buffer, we need to load more data
                self.buflen = try_ready! {self.r.poll_read(&mut self.buf)}; //try_ready macro is equal to bottom 3 three lines. Handling errors and notReady case
                // self.buflen = match self.r.poll_read(&mut self.buf)?{
                //     Async::Ready(n) => n,
                //     Async::NotReady => return Ok(Async::NotReady),
                // };
                self.ptr = 0;
            }
            if self.buflen == 0{ //No more data, finish
                return Ok(Async::Ready((None)));
            }
            match self.reading{
                Rding::Len(ref mut nb) => {
                    match self.buf[self.ptr]{ //we can see ":" or number in the length mode => like 125:Hello
                        b':' => self.reading = Rding::Val(*nb,Vec::new()), //cahnge reading length mode to reading text mode
                        v if v >= b'0' && v <= b'9' => {
                            *nb = *nb * 10 + ((v-48) as usize); //0 in asci 48
                        }
                        _ => {}
                    }
                    self.ptr += 1;
                },
                Rding::Val(n, ref mut v) => { //v is vector that we already put together //n is byte nuöber should be read
                    let p_dist = std::cmp::min(self.ptr+n-v.len(),self.buflen); //we can not exceed buffer, so limit with bufferlength.
                    v.append(&mut self.buf[self.ptr..p_dist].to_vec());
                    self.ptr = p_dist;
                    if v.len() == n{
                        let res = String::from_utf8(v.clone())?;
                        self.reading = Rding::Len(0);
                        return Ok(Async::Ready(Some(res)));
                    }
                }
            }
        }

        Ok(Async::Ready(Some("Hello".to_string())))
    }
}