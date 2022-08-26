use reqwest;
use postgres;
use serde::{Deserialize, Serialize};
use std::env;

// !Esquemas para serializar la informacion de las naciones
#[derive(Serialize, Deserialize, Debug)]
struct Nation {
    pub id: i32,
    pub name: Option<String>,
}

// !Esquemas para serializar la informacion de las equipos
#[derive(Serialize, Deserialize, Debug)]
struct Club {
    pub id: i32,
    pub name: Option<String>,
}

// !Esquemas para serializar la informacion de los jugadores
#[derive(Serialize, Deserialize, Debug)]
struct Player {
    pub id: i32,
    pub name: Option<String>,
    pub position: Option<String>,
    pub nation: Option<i32>,
    pub club: Option<i32>
}

// !Esquemas para serializar la respuesta de la api, agregando un generico
#[derive(Serialize, Deserialize, Debug)]
struct Response<T> {
    pub count: i32,
    pub count_total: i32,
    pub page: i32,
    pub page_total: i32,
    pub items_per_page: i32,
    pub items: Vec<T>
}

#[tokio::main]
async fn main() {
    //? Se intancia el servicio para realizar las peticiones
    let client_request = reqwest::Client::new();

    //TODO Se captura el valor de las variables de entorno
    let driver = env::var("DATABASE_DRIVER").unwrap();
    let user = env::var("DATABASE_USER").unwrap();
    let password = env::var("DATABASE_PASSWORD").unwrap();
    let port = env::var("DATABASE_PORT").unwrap();
    let database = env::var("DATABASE_DATABASE").unwrap();

    //* Se crea cadena que establece la conexion con la base de datos 
    let uri = format!("{}://{}:{}@database:{}/{}", driver, user, password, port, database);

    // !Se crea la conexion con la base de datos
    let mut client_postgres = postgres::Client::connect(&uri, postgres::NoTls).unwrap();
    
    //? Se crea un arregla para hacer las diferentes peticiones de los endpoints que se busca obtener la informacion
    let options = ["nations", "clubs", "players"];

    for option in options {
        let url = format!("https://futdb.app/api/{}", option);
        //TODO Se captura el valor del TOKEN para enviarlo por los header de la peticion HTTP
        let token = env::var("TOKEN").unwrap();
        // !Se realiza la primera peticion con la intencion de conocer la cantidad de paginas
        let first_response = client_request
            .get(url)
            .header("X-AUTH-TOKEN", token)
            .send()
            .await
            .unwrap();
        //* Se serializa la respuesta, en esta caso es inecesario el tipado, ya que solo se busca la cantidad de paginas
        let first_data = first_response.json::<Response<Nation>>().await.unwrap();
        
        //? Se itera las paginas iniciando desde 1
        for page in 1..=first_data.items_per_page {
            let url = format!("https://futdb.app/api/{}", option);
            let token = env::var("TOKEN").unwrap();
            let response = client_request
                .get(url)
                .query(&[("page", page)])
                .header("X-AUTH-TOKEN", token)
                .send()
                .await
                .unwrap();
            // !Se evalua la opcion de la peticion para serializar correctamente la respuesta de la api
            // !y se realiza el almacenamiento en la base de datos
            if option == "nations" {
                let data = response.json::<Response<Nation>>().await.unwrap();
        
                for item in data.items {
                    client_postgres.execute(
                        "INSERT INTO nations(id, name) VALUES ($1, $2)",
                        &[&item.id, &item.name]
                    ).unwrap();
                }
            } else if option == "clubs" {
                let data = response.json::<Response<Club>>().await.unwrap();
        
                for item in data.items {
                    client_postgres.execute(
                        "INSERT INTO clubs(id, name) VALUES ($1, $2)",
                        &[&item.id, &item.name]
                    ).unwrap();
                }
            } else {
                let data = response.json::<Response<Player>>().await.unwrap();
        
                for item in data.items {
                    client_postgres.execute(
                        "INSERT INTO players(id, name, position, nation_id, club_id) VALUES ($1, $2, $3, $4, $5)",
                        &[&item.id, &item.name, &item.position, &item.nation, &item.club]
                    ).unwrap();
                }
            }
    
        }
    }

    
}
