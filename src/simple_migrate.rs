use mysql;

pub struct Migration {
    name: String,
    up: String,
    down: String,
}

pub struct Migrations {
    conn_string: String,
    migrations_wanted: Vec<Migration>,
}

impl Migrations {
    pub fn new(connection_string: &str) -> Migrations {
        Migrations {
            conn_string: connection_string.to_string(),
            migrations_wanted: vec![],
        }
    }

    fn ensure_tables(&self) {
        let pool: mysql::Pool = mysql::Pool::new(self.conn_string.clone()).unwrap();
        pool.prep_exec(
            "CREATE TABLE IF NOT EXISTS __migrations(id INT NOT NULL AUTO_INCREMENT PRIMARY KEY, name TEXT NOT NULL, up TEXT NOT NULL, down TEXT NOT NULL);",
            (),
        )
        .unwrap();
    }

    fn get_applied_migrations(&self) -> Vec<Migration> {
        let mut list = vec![];
        let pool: mysql::Pool = mysql::Pool::new(self.conn_string.clone()).unwrap();
        let results = pool
            .prep_exec(
                "SELECT id,name,up,down FROM __migrations ORDER BY name;",
                (),
            )
            .unwrap();
        for result in results {
            for mut row_ in result {
                let item = Migration {
                    name: row_.take("name").unwrap(),
                    up: row_.take("up").unwrap(),
                    down: row_.take("down").unwrap(),
                };
                list.push(item);
            }
        }
        list
    }

    fn insert_db_migration(&self, name: &str, up: &str, down: &str) {
        let pool: mysql::Pool = mysql::Pool::new(self.conn_string.clone()).unwrap();
        pool.prep_exec(
            "INSERT INTO __migrations(name, up , down) VALUES (:name,:up,:down);",
            params! {
                "name" => name,
                "up" => up,
                "down" => down,
            },
        )
        .unwrap();
    }

    pub fn add_migration(&mut self, name: &str, up: &str, down: &str) {
        let m = Migration {
            name: name.to_string(),
            up: up.to_string(),
            down: down.to_string(),
        };
        self.migrations_wanted.push(m);
        self.migrations_wanted
            .sort_unstable_by(|a, b| b.name.cmp(&a.name));
    }

    fn apply_migration(&self, migration: &Migration) {
        let pool: mysql::Pool = mysql::Pool::new(self.conn_string.clone()).unwrap();
        println!("APPLY UP '{}'", migration.up);
        pool.prep_exec(&migration.up, ()).unwrap();
        self.insert_db_migration(&migration.name, &migration.up, &migration.down);
    }

    fn unapply_migration(&self, migration: &Migration) {
        let pool: mysql::Pool = mysql::Pool::new(self.conn_string.clone()).unwrap();
        println!("APPLY DOWN'{}'", migration.down);
        pool.prep_exec(&migration.down, ()).unwrap();
        self.insert_db_migration(&migration.name, &migration.up, &migration.down);
    }

    pub fn do_migrations(&self) {
        self.ensure_tables();

        let migrations_applied = self.get_applied_migrations();
        // apply all migrations, that are not applied
        for wanted in self.migrations_wanted.iter() {
            let mut found = false;
            for applied in migrations_applied.iter() {
                if applied.name == wanted.name {
                    found = true;
                }
            }
            if !found {
                self.apply_migration(&wanted);
            }
        }

        // unapply all migrations, that are not in wanted
        for wanted in migrations_applied.iter().rev() {
            let mut found = false;
            for applied in self.migrations_wanted.iter() {
                if applied.name == wanted.name {
                    found = true;
                }
            }
            if !found {
                self.unapply_migration(&wanted);
            }
        }
    }
}
