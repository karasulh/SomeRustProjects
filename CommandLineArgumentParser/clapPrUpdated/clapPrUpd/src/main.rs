#![feature(proc_macro_hygiene)]
use clap::{clap_app,crate_version}; //command line argument parser=clap
use pulldown_cmark::{html::push_html, Event, Parser}; //pull parser for markdown
use maud::html;//to write HTML templates

fn wrap_html(s:&str, css:Option<&str>)->String{
    let res = html!{
        (maud::DOCTYPE)
        html{
            head{
                meta charset = "utf-8";
                @if let Some(s) = css{
                    link rel = "stylesheet" type="text/css" href=(s) {}
                }
            }
            body{
                (maud::PreEscaped(s))
            }
        }
    };
    res.into_string()
}

fn main() {
    
    let clap = clap_app!( clapPr =>
                            (version:crate_version!())
                            (author:"Bugrahan Kara")
                            (about:"Renders markdown as you like") 
                            (@arg inputUser: +required "Sets the input file") //if the user must give an input, put "required"
                            (@arg wrap: -w "Wrap in html")
                            (@arg event: -e "Print event")
                            (@arg css: --css + takes_value "Link to css" )
    ).get_matches();

    println!("Input = {:?}", clap.value_of("inputUser")); //Option value of input

    let infile = std::fs::read_to_string(clap.value_of("inputUser").unwrap()).expect("Could not read file");

    let ps = Parser::new(&infile);

    let ps:Vec<Event> = ps.into_iter().collect(); //Each different writing styles in markdown creates a event when parsing, so take them into vector
    if clap.is_present("event"){
        for p in &ps{
            println!("{:?}",p); //See this events
        }
    }
    
    let mut res = String::new();
    push_html(&mut res, ps.into_iter()); //it returns markdown parsing events to html format

    if clap.is_present("wrap"){
        res = wrap_html(&res, clap.value_of("css"));
    }

    println!("Done\n");
    println!("{}",res);
}

//1-Run it with:
//cargo run -- --help ==> it shows the information about project which were given above
//cargo run -- hello.world ==> user gives a input, hello.world=inputUser

//2-Run it with:
//cargo run -- test_data\hello.md ==> it takes markdown file and returns it with html format

//3-Run it with after calling "rustup install nightly"
//cargo +nightly run --test_data/hello.md -w --css foo.css