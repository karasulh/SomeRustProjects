use actix::prelude::*;
use actix_web::HttpRequest;
use actix_web ::{fs, server::HttpServer,App,HttpResponse,Json, ws::HandshakeError, handler::AsyncResponder,Query,State,Responder};
use actix_web::middleware::session::{CookieSessionBackend,RequestSession,SessionStorage};
use shop_base::{Conn,Item};
use failure::Error;
use futures::Future;
use serde_derive::*;
use handlebars::Handlebars;
//HTTP Server create several apps of the same type and run them simultaneously
//For find_items, it will connect database and find the searched term. Before it, write this "cp ../shop_base/.env ./" to termianal for enviroment variables to access db 
//Server take closure instead of a list of apps because the serer will create several instances of each app and the number depends on cores computer has.
//In Actix, resource is a section of an app that can handle specific type of request
//We use DBExecutor instead of creating a new connection to allow our server to accept many connections while still waiting for a Database response. 
//Future implements responder as it's own result type, by returning the future, we get the result from our DB future back to the user.
//"f" takes a closure with a HttpRequest, but "with" takes a closure an extractor.
//Middleware works on request, change request before it gets to the main resources.

enum DBRequest{
    FindItems(String,i64),
}

enum DBResponse{
    FoundItems(Vec<Item>),
}

impl Message for DBRequest{
    type Result = Result<DBResponse,Error>;
}

pub struct DBExecutor{
    conn: Conn,
}

impl Actor for DBExecutor{ //system can treat as an actor and send it messages and know how to handle the response 
    type Context = SyncContext<Self>;
}

impl Handler<DBRequest> for DBExecutor {
    type Result = Result<DBResponse,Error>;
    fn handle(&mut self, msg:DBRequest,_:&mut Self::Context) -> Self::Result{
        match msg{
            DBRequest::FindItems(s,i)=> Ok(DBResponse::FoundItems(self.conn.find_items(&s,i)?)),
        }
    }
}

#[derive(Deserialize,Debug)]
struct FormFindItems{
    search_term:String,
    limit:Option<i64>,
}

fn search<F>(page_hand: &Addr<DBExecutor>,ffi:&FormFindItems,req:&HttpRequest<F>)->impl Responder{

    let searches = req.session().get::<i32>("searches").expect("Session should exist").unwrap_or(0) + 1;
    req.session().set("searches",searches).expect("Coudlnot set searches");

    page_hand
    .send(DBRequest::FindItems(ffi.search_term.clone(), ffi.limit.unwrap_or(5)))
    .and_then(move |r| match r{
        Ok(DBResponse::FoundItems(v)) => Ok(HttpResponse::Ok()//Ok(HttpResponse::Ok().json(v)),
            .content_type("text/html")
            .body(TEMPLATES.render("item_list",&(&v,searches)).unwrap())),
        Err(_)=> Ok(HttpResponse::Ok().json("Error finding database")),
    })
    .responder() //responder is a future
}

lazy_static::lazy_static! {
    static ref TEMPLATES:Handlebars = {
        let mut res = Handlebars::new();
        let df = std::fs::read_to_string("test_site/templates/item_list.html").expect("Couldnot read template");
        res.register_template_string("item_list",df).expect("Coudlnot parse template");
        res
    };
}

fn main() {
    let sys = System::new("shop_site");

    //3 active connection to the db
    let db_hand = SyncArbiter::start(3, || DBExecutor{  //we need to run some DBExecutors
        conn: Conn::new().unwrap(),
    });

    HttpServer::new(move|| {
        vec![
            //App::new()
            App::with_state(db_hand.clone())
            .prefix("/db/")
            .middleware(SessionStorage::new(CookieSessionBackend::signed(&[0;32]).secure(false),
            ))
            .resource("/",|r|{ //localhost::8088/db/
                r.f(|_|{
                    println!("DB Creating");
                    HttpResponse::Ok().content_type("text/plain").body("This is the database side of the app")
                })
            })
            .resource("/find_items", |r|{ //localhost::8080/db/find_items?search_term="ca"
                //r.f(|req| {//-> impl Responder{ //-> Result<_,Error> {
                r.method(http::Method::GET).with(|(state,query,req):(State<Addr<DBExecutor>>,Query<FormFindItems>,HttpRequest<_>)| { //more specific, only get POST method 
                    // let st=req
                    //     .query()
                    //     .get("search_term")
                    //     .map(|x|x.clone())
                    //     .unwrap_or("".to_string());
                    //let conn = Conn::new()?;
                    //req.state()

                    // state
                    //     //.send(DBRequest::FindItems(st, 5))
                    //     .send(DBRequest::FindItems(query.search_term.clone(), query.limit.unwrap_or(5)))
                    //     .and_then(|r| match r{
                    //         Ok(DBResponse::FoundItems(v)) => Ok(HttpResponse::Ok().json(v)),
                    //         Err(_)=> Ok(HttpResponse::Ok().json("Error finding database")),
                    //     })
                    //     .responder() //responder is a future

                    //let items = conn.find_items(&st,5)?;
                    //Ok(Json(items))

                    search(&state,&form,&req)
                });

                r.method(http::Method::POST).with(|(state,query,req):(State<Addr<DBExecutor>>,Query<FormFindItems>,HttpRequest<_>)| { //more specific, only get GET method
                    search(&state, &query,&req)
                })
            })
            //.finish(),
            .boxed(),//when we have more than 1 type of app with different state, use "boxed"

            App::new()
            .handler("/",fs::StaticFiles::new("test_site/static/").unwrap()
            .show_files_listing()
            .index_file("index.html"))
            //.finish()
            .boxed()
        ]}
    )
    .bind("127.0.0.1:8088").unwrap()
    //.run();//if there is no system, then run server
    .start(); //if there is system, start server

    sys.run();


    println!("Done");
}
