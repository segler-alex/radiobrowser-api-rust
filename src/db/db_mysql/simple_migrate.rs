use mysql;

pub struct Migration {
    name: String,
    up: String,
    down: String,
}

pub struct Migrations <'a> {
    pool: &'a mysql:: Pool,
    migrations_wanted: Vec<Migration>,
}

impl <'a>Migrations <'a> {
    pub fn new(pool: &mysql::Pool) -> Migrations {
        Migrations {
            pool: pool,
            migrations_wanted: vec![],
        }
    }

    fn ensure_tables(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.pool.prep_exec(
            "CREATE TABLE IF NOT EXISTS __migrations(id INT NOT NULL AUTO_INCREMENT PRIMARY KEY, name TEXT NOT NULL, up TEXT NOT NULL, down TEXT NOT NULL);",
            (),
        )?;
        Ok(())
    }

    fn get_applied_migrations(&self) -> Result<Vec<Migration>, Box<dyn std::error::Error>> {
        let mut list = vec![];
        let results = self.pool
            .prep_exec(
                "SELECT id,name,up,down FROM __migrations ORDER BY name;",
                (),
            )?;
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
        Ok(list)
    }

    fn insert_db_migration(&self, name: &str, up: &str, down: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.pool.prep_exec(
            "INSERT INTO __migrations(name, up , down) VALUES (:name,:up,:down);",
            params! {
                name, up, down,
            },
        )?;
        Ok(())
    }

    fn delete_db_migration(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.pool.prep_exec(
            "DELETE FROM __migrations WHERE name=:name;",
            params! {
                name,
            },
        )?;
        Ok(())
    }

    pub fn add_migration(&mut self, name: &str, up: &str, down: &str) {
        let m = Migration {
            name: name.to_string(),
            up: up.to_string(),
            down: down.to_string(),
        };
        self.migrations_wanted.push(m);
        self.migrations_wanted
            .sort_unstable_by(|a, b| a.name.cmp(&b.name));
    }

    fn apply_migration(&self, migration: &Migration, ignore_errors: bool) -> Result<(),Box<dyn std::error::Error>> {
        info!("APPLY UP '{}'", migration.name);
        let result = self.pool.prep_exec(&migration.up, ());
        match result {
            Err(err) => {
                if !ignore_errors {
                    panic!(err.to_string());
                }
            },
            _ => {}
        };
        self.insert_db_migration(&migration.name, &migration.up, &migration.down)?;
        Ok(())
    }

    fn unapply_migration(&self, migration: &Migration, ignore_errors: bool) -> Result<(),Box<dyn std::error::Error>> {
        info!("APPLY DOWN '{}'", migration.name);
        let result = self.pool.prep_exec(&migration.down, ());
        match result {
            Err(err) => {
                if !ignore_errors {
                    panic!(err.to_string());
                }
            },
            _ => {}
        };
        self.delete_db_migration(&migration.name)?;
        Ok(())
    }

    pub fn do_migrations(&self, ignore_errors: bool, allow_database_downgrade: bool) -> Result<(),Box<dyn std::error::Error>> {
        self.ensure_tables()?;

        let migrations_applied = self.get_applied_migrations()?;
        // apply all migrations, that are not applied
        for wanted in self.migrations_wanted.iter() {
            let mut found = false;
            for applied in migrations_applied.iter() {
                if applied.name == wanted.name {
                    found = true;
                }
            }
            if !found {
                self.apply_migration(&wanted, ignore_errors)?;
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
                if allow_database_downgrade {
                    self.unapply_migration(&wanted, ignore_errors)?;
                }else{
                    panic!("Database downgrade would be neccessary! Please confirm if you really want to do that.")
                }
            }
        }

        Ok(())
    }
}
