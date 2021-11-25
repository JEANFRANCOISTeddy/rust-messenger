use std::fs::read_to_string;
use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use std::io::stdin;
use std::error::Error;

use mongodb::bson::{doc, Document};
use mongodb::{options::ClientOptions, Client};
use serde::{Serialize, Deserialize};
use warp::{http::StatusCode, Filter};
use tokio;

const MSG_SIZE: usize = 50;

enum Gender {
    Male,
    Female,
    Other,
}

struct Person {
    name: String,
    gender: Gender,
    password: String,
}

#[tokio::main]
async fn main() -> mongodb::error::Result<()>  {
    let mut dbclient = Client::with_uri_str("mongodb://127.0.0.1:27017").await?;
    init_db(dbclient.clone());

    let mut person: Person = register();
    create_person(dbclient, person.name, person.password).await?;

    let mut client = TcpStream::connect("localhost:8080").expect("Connexion Failed");

    client
        .set_nonblocking(true)
        .expect("failed to initiate non-blocking");

    let (sending, receive) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Charset utf8 error");
                println!("------------------------------------");

                println!("Message  : {:?}", msg);
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("connexion avec le serveur Stop");
                break;
            }
        }

        match receive.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("Ecriture Socket Error");
                println!("message senvoyé {:?}", msg);
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a Message:");
    loop {
        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("reading from stdin failed");
        let msg = buff.trim().to_string();
        if msg == ":quit" || sending.send(msg).is_err() {
            break;
        }
    }
    println!("bye bye!");

    Ok(())
}

async fn init_db(dbclient: Client) -> mongodb::error::Result<()> {
    let database = dbclient.database("local").run_command(doc! {"ping": 1}, None).await?;
    println!("Connected successfully");
    Ok(())
}

fn register() -> Person {
    let mut line = String::new();

    let mut line3 = String::new();

    println!("Qu'elle est votre nom:");
    std::io::stdin().read_line(&mut line).unwrap();

    let line2 = gender();

    println!("Créer votre Password:");
    std::io::stdin().read_line(&mut line3).unwrap();
    println!("Hello , {}", line);

    let p = Person {
        name: line,
        gender: line2,
        password: line3,
    };
    return p;
}

fn gender() -> Gender {
    let male = "Male".to_string();
    let female = "Female".to_string();
    let other = "Other".to_string();
    
    
    let gen = loop {
        let mut line2 = String::new();

        
        println!("Qu'elle est votre Genre (Male,Female,Other):");
        stdin().read_line(&mut line2).expect("Failed to read input.");
        let line2 = line2.lines().next().expect("Could not read entry.").trim_right();;
       
        
        println!("{}", line2);
        println!("{}", line2.eq(&male));

        if line2.eq(&male) == true {
            let gen = Gender::Male;
            println!("test");
            break gen;
        } else if female == line2 {
            let gen = Gender::Female;
            break gen;
        } else if other == line2 {
            let gen = Gender::Other;
            break gen;
        }
    };

    return gen;

    /*
    let mut line2 = String::new();
    println!("Qu'elle est votre Genre (Male,Female,Other):");
    std::io::stdin().read_line(&mut line2).unwrap();

    match line2 {
        male if male == line2 => {
            let gen = Gender::Male;
            return gen;
        }
        female if female == line2 => {
            let gen = Gender::Female;

            return gen;
        }
        other => {
            let gen = Gender::Other;
            return gen;
        }
        _ => println!("Bonne typo attendu"),
    }*/
}

async fn create_person(dbclient: Client, _name: String, _password: String) -> mongodb::error::Result<()> {
    let db = dbclient.database("local");
    let coll = db.collection("persons");

    let person = doc! { 
        "name": _name, 
        "password": _password
    };

    coll.insert_one(person, None).await?;

    println!("Person has been created");

    Ok(())
}
