extern crate shop_base;
use shop_base::Conn;
use failure::Error;

fn main()->Result<(),Error>{

    let clap = clap_app!(blog_cli => 
        (about:"Edit the shop_base contents")
        (version: crate_version!())
        (author: "MBK")
        (@subcommand put => 
            (about: "Put an item on the database")
            (@arg name: +required "The name of the item")
            (@arg description: +required "The description of the item")
            (@arg price: +required "The price of the item in pence")
        )
        (@subcommand find => 
            (about: "Find items matching a given name part")
            (@arg name: +required "A partial match of the item name")
            (@arg limit: +takes_value "The max number of entries to return")
        )
        (@subcommand stock => 
            (about: "Set the stock level for an item in the store")
            (@arg id: +required "The item id")
            (@arg amount: +required "The new value")
            (@arg rel:-r "Add to previous value")
        )
    )
    .get_matches();

    if let Some(sub) = clap.subcommand_matches("put"){
        let r = conn.put_item(
            sub.value_of("name").unwrap(),
            sub.value_of("description").unwrap(),
            sub.value_of("price").and_then(|v|v.parse().ok()).unwrap(),
        )?;
        println!("Added Item {:?}",r);
    }

    if let Some(sub) = clap.subcommand_matches("find"){
        let r = conn.find_items(
            sub.value_of("name").unwrap(),
            sub.value_of("limit").and_then(|v|v.parse().ok()).unwrap_or(5),
        )?;
        for p in r {
            println!("\n ---- Entry ---- \n");
            println!("{:?}",p);
        }
    }

    if let Some(sub) = clap.subcommand_matches("stock"){
        let r = conn.set_stock(
            sub.value_of("id").unwrap().parse().unwrap(),
            sub.value_of("amount").unwrap().parse().unwrap(),
            sub.is_present("rel"),
        )?;
        println!("Updated {:?}",r);
    }

    // let conn = Conn::new();
    // let r = conn.put_item("Car","Drives around a bit",300)?;
    // println!("Added Item {:?}",r);
    Ok(())
}