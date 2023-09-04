use clap::{clap_app, crate_version};
use regex::Regex;
use std::path::Path;
use failure::{Error,Fail};
use std::fmt;

//Extensible Generic Errors with Failure Crate
//The type which is used with Fail trait, must be require "Display, Debug, Send+Sync(thread)" traits.

// #[derive(Debug)]
// struct ArgErr{ 
//     arg: &'static str,
// }

// impl fmt::Display for ArgErr{
//     fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result{
//         write!(f,"Argument Not Provided: {}",self.arg)
//     }
// }

// impl Fail for ArgErr{}

//Another way of the top part, look at the documentation of Fail crate
#[derive(Debug,Fail)]
#[fail(display = "Argument Not Provided '{}'",arg)] 
struct ArgErr{ 
    arg: &'static str,
}

#[derive(Debug)]
struct Record {
    line: usize,
    tx: String,
}

//fn process_file<P: AsRef<Path>>(p: P, re: Regex) -> Result<Vec<Record>, String> {
fn process_file<P: AsRef<Path>>(p: P, re: &Regex) -> Result<Vec<Record>, Error> {
    let mut res = Vec::new();
    //let bts = std::fs::read(p).map_err(|e| "couldnot read string".to_string())?;
    let bts = std::fs::read(p)?;


    if let Ok(ss) = String::from_utf8(bts) {
        for (i, l) in ss.lines().enumerate() {
            if re.is_match(l) {
                res.push(Record {
                    line: i,
                    tx: l.to_string(),
                });
            }
        }
    }

    return Ok(res);
}

//To check whether the path is file or directory because if it is directory, potentially search many files
fn process_path<P,FF,EF>(p: P, re: &Regex, ff: &FF, ef: &EF)->Result<(),Error> 
    where P:AsRef<Path>, 
    FF:Fn(&Path,Vec<Record>),
    EF: Fn(Error), //if the first file returns error, we couldnot look other files in directory, so we use this method to handle this case.
{

    let p = p.as_ref();
    let md = p.metadata()?; //if the file doesnot exist, or problematic then it returns error.
    let ft = md.file_type();
    if ft.is_file(){
        let dt = process_file(p, re)?;
        ff(p,dt);
    }

    if ft.is_dir(){
        let dd = std::fs::read_dir(p)?;
        for d in dd{
            //let r = process_path(d?.path(), re, ff, ef)?; //p?.path() => check firstly metadata of p, then get path
            if let Err(e) = process_path(d?.path(), re, ff, ef){
                ef(e);
            }
        }
    }


    Ok(())
}

//fn main() -> Result<(), String> {
//fn main() -> Result<(), Error> {
fn main(){
    if let Err(e) = run(){
        println!("There was an error: {}",e); //more well written error
    }
}



fn run() -> Result<(),Error>{
    let cp = clap_app!(
        pgrep =>
        (version: crate_version!())
        (about: "A Grep like program")
        (author: "Bugra")
        (@arg pattern: +required "The regex pattern to search for")
        (@arg file: -f --file +takes_value "The file to test")
    )
    .get_matches();

    //let re = Regex::new(cp.value_of("pattern").unwrap()).map_err(|_|"Bad regex pattern")?;
    let re = Regex::new(cp.value_of("pattern").unwrap())?;

    //let p = process_file(cp.value_of("file").ok_or("No file chosen")?,&re);
    //let p = process_file(cp.value_of("file").ok_or(ArgErr{arg:"file"})?,&re);
    let p = process_path(
        cp.value_of("file").ok_or(ArgErr{arg:"file"})?,
        &re,
        &|pt,v|{
        println!("{:?}",pt);
        println!("{:?}",v);
        },
        &|e|{
            println!("Error:{}",e);
        });


    println!("{:?}", p);
    Ok(())
}

//cargo run -- Hello -f test_data/t1.txt  //=> To find "Hello" word in t1.txt
//cargo run Hello => gives error: "Error: ArgErr{arg:file}" (in Video 7)


//(?)When we are simply passing errors back up the stack with, we should use failure::Error.
//(?)When the caller needs to make a decision based on the kind of error and when our function creates the error itself, we shouldnot return failure::Error.
//Handling not fatal errors: receive a closure and call that with each error and return a vec of errors.
//The main advantage of Error type: It avoids creating a new error type for every situation