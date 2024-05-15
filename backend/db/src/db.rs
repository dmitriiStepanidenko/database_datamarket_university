use assert_fs::TempDir;
use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;
use std::fs;
use surrealdb::opt::auth::Root;
use surrealdb::{
    dbs::Capabilities,
    engine::any::Any,
    opt::Config,
    //Connection
    //opt::{Config, IntoEndpoint},
    //opt::Endpoint,
    Surreal,
};
use surrealdb_migrations::MigrationRunner;

#[cfg(debug_assertions)]
lazy_static::lazy_static! {
    static ref DB_ADDR: String = "ws://localhost:8000".to_string();
    static ref DB_NAMESPACE: String = "namespace_test".to_string();
    static ref DB_DATABASE: String = "database_test".to_string();
}
static DB_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR");

#[cfg(not(debug_assertions))]
lazy_static::lazy_static! {
    static ref DB_ADDR: String =
        std::env::var("DB_ADDR").expect("DB_ADDR must be set");
    static ref DB_NAMESPACE: String =
        std::env::var("DB_NAMESPACE").expect("DB_NAMESPACE must be set");
    static ref DB_DATABASE: String =
        std::env::var("DB_DATABASE").expect("DB_DATABASE must be set");
    static ref ADMIN_USER_EMAIL: String =
        std::env::var("ADMIN_USER_EMAIL").expect("SHOULD PROVIDE ADMIN_USER_EMAIL");
    static ref ADMIN_PASSWORD: Option<String> =
        std::env::var("ADMIN_PASSWORD").ok();
}

#[cfg(not(test))]
pub type Db = Surreal<Any>;
#[cfg(test)]
pub type Db = Surreal<Any>;

pub static DB: Lazy<Db> = Lazy::new(Surreal::init);

pub async fn set_test_db() -> Db {
    //use tracing_log::LogTracer;
    //let _ = LogTracer::init();
    static ONCE_LOG: std::sync::Once = std::sync::Once::new();
    ONCE_LOG.call_once(|| {
        tracing_subscriber::fmt::init();
    });

    let mut config = Config::default().capabilities(Capabilities::all());
    config = config.user(Root {
        username: "root",
        password: "root",
    });

    let db: Db = Surreal::init();
    db.connect(("memory", config))
        .await
        .expect("Can't connect to local (Mem) db");
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await
    .unwrap();
    db.use_ns(DB_NAMESPACE.as_str())
        .use_db(DB_DATABASE.as_str())
        .await
        .expect("Problem with namespace or database");
    let version = db.version().await.expect("Could not get db version!");
    println!("->> DB version: {version}");
    println!("DIR:{:?}\n\n", DB_DIR);

    let temp_dir = TempDir::new().unwrap();
    DB_DIR.extract(temp_dir.path()).unwrap();

    let config_file_path = temp_dir.join(".surrealdb");

    let content = format!(
        r#"
    [core]
    path = "{}"

    [db]
    username = "root"
    password = "root"
    ns = "{}"
    db = "{}""#,
        temp_dir.display(),
        DB_NAMESPACE.as_str(),
        DB_DATABASE.as_str()
    );
    fs::write(config_file_path.clone(), content).unwrap();

    let runner = MigrationRunner::new(&db);
    runner
        .use_config_file(&config_file_path)
        .up()
        .await
        .expect("Failed to apply migrations");
    db
}

#[cfg(test)]
pub async fn set_database() {
    //let db = Surreal::new::<Mem>(())
    //    .await
    //    .expect("Error while creating db in mem");
    //let connection = db
    //    .connect::<Mem>(())
    //    .await
    //    .expect("Error while connecting to db in mem");
    //let version = db.version().await.expect("Could not get db version!");
    //println!("->> DB version: {version}");
    // Apply all migrations
    use tracing::Level;
    use tracing_log::LogTracer;
    use tracing_subscriber;
    let _collector = tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::TRACE)
        // build but do not install the subscriber.
        .finish();
    let _ = LogTracer::init();
    DB.connect("memory")
        .await
        .expect("Can't connect to local (Mem) db");
    DB.use_ns(DB_NAMESPACE.as_str())
        .use_db(DB_DATABASE.as_str())
        .await
        .expect("Problem with namespace or database");
    let version = DB.version().await.expect("Could not get db version!");
    println!("->> DB version: {version}");
    set_migrations().await;
}

#[cfg(debug_assertions)]
pub async fn set_migrations() {
    let remove_db = match std::env::var("REMOVE_DB") {
        Ok(val) => !val.is_empty() && val != "0" && val.to_lowercase() != "false",

        Err(_) => false,
    };

    if remove_db {
        DB.query(format!("REMOVE DATABASE {};", DB_DATABASE.as_str()))
            .await
            .expect("Problem with removing DB");
    }
    DB.signin(Root {
        username: "root",
        password: "root",
    })
    .await
    .expect("Cant log in DB");
    DB.use_db(DB_DATABASE.as_str())
        .await
        .expect("Problem with setting database");
    //const DB_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR");
    println!("DIR:{:?}\n\n", DB_DIR);
    println!("DIR:{:?}\n\n", DB_DIR);
    MigrationRunner::new(&DB)
        .load_files(&DB_DIR)
        .up()
        .await
        .expect("Failed to apply migrations");
}

#[cfg(not(debug_assertions))]
pub async fn set_migrations() {
    DB.use_db(DB_DATABASE.as_str())
        .await
        .expect("Problem with setting database");

    println!("DIR:{:?}\n\n", DB_DIR);
    let temp_dir = TempDir::new().unwrap();
    DB_DIR.extract(temp_dir.path()).unwrap();

    let config_file_path = temp_dir.join(".surrealdb");

    let content = format!(
        r#"
    [core]
    path = "{}"

    [db]
    address = "{}"
    username = "root"
    password = "root"
    ns = "{}"
    db = "{}""#,
        temp_dir.display(),
        DB_DATABASE.as_str(),
        DB_NAMESPACE.as_str(),
        DB_DATABASE.as_str()
    );
    fs::write(config_file_path.clone(), content).unwrap();

    let runner = MigrationRunner::new(&DB);
    runner
        .use_config_file(&config_file_path)
        .up()
        .await
        .expect("Failed to apply migrations");
}

#[cfg(not(test))]
pub async fn set_database() {
    DB.connect(DB_ADDR.as_str())
        .await
        .expect("Can't connect to db");
    let version = DB.version().await.expect("Could not get db version!");
    println!("->> DB version: {version}");

    // Select a specific namespace / database
    DB.use_ns(DB_NAMESPACE.as_str())
        .use_db(DB_DATABASE.as_str())
        .await
        .expect("Problem with namespace or database");
    // Apply all migrations

    set_migrations().await;
}
