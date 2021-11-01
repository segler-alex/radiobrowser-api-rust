use mysql::TxOpts;
use mysql;
use mysql::prelude::*;

pub struct Migration {
    name: String,
    up: String,
    down: String,
}

pub struct Migrations<'a> {
    pool: &'a mysql::Pool,
    migrations_wanted: Vec<Migration>,
}

impl<'a> Migrations<'a> {
    pub fn new(pool: &mysql::Pool) -> Migrations {
        Migrations {
            pool: pool,
            migrations_wanted: vec![],
        }
    }

    fn ensure_tables(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.pool.get_conn()?.query_drop(
            "CREATE TABLE IF NOT EXISTS __migrations(id INT NOT NULL AUTO_INCREMENT PRIMARY KEY, name TEXT NOT NULL, up TEXT NOT NULL, down TEXT NOT NULL);")?;
        Ok(())
    }

    fn get_applied_migrations(&self) -> Result<Vec<Migration>, Box<dyn std::error::Error>> {
        let list = self.pool.get_conn()?.query_map(
            "SELECT name,up,down FROM __migrations ORDER BY name;",
            |(name, up, down)| Migration { name, up, down },
        )?;
        Ok(list)
    }

    fn insert_db_migration(
        &self,
        conn: &mut mysql::Transaction,
        name: &str,
        up: &str,
        down: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        conn.exec_drop(
            "INSERT INTO __migrations(name, up , down) VALUES (:name,:up,:down);",
            params! {
                name, up, down,
            },
        )?;
        Ok(())
    }

    fn delete_db_migration(
        &self,
        conn: &mut mysql::Transaction,
        name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        conn.exec_drop(
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

    fn apply_migration(
        &self,
        conn: &mut mysql::PooledConn,
        migration: &Migration,
        ignore_errors: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("APPLY UP '{}'", migration.name);
        let mut conn = conn.start_transaction(TxOpts::default())?;
        let result = conn.query_drop(&migration.up);
        match result {
            Err(err) => {
                if !ignore_errors {
                    panic!("{}", err.to_string());
                }
            }
            _ => {}
        };
        self.insert_db_migration(&mut conn, &migration.name, &migration.up, &migration.down)?;
        conn.commit()?;
        Ok(())
    }

    fn unapply_migration(
        &self,
        conn: &mut mysql::PooledConn,
        migration: &Migration,
        ignore_errors: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("APPLY DOWN '{}'", migration.name);
        let mut conn = conn.start_transaction(TxOpts::default())?;
        let result = conn.query_drop(&migration.down);
        match result {
            Err(err) => {
                if !ignore_errors {
                    panic!("{}", err.to_string());
                }
            }
            _ => {}
        };
        self.delete_db_migration(&mut conn, &migration.name)?;
        conn.commit()?;
        Ok(())
    }

    pub fn do_migrations(
        &self,
        ignore_errors: bool,
        allow_database_downgrade: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.ensure_tables()?;
        let mut conn = self.pool.get_conn()?;

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
                self.apply_migration(&mut conn, &wanted, ignore_errors)?;
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
                    self.unapply_migration(&mut conn, &wanted, ignore_errors)?;
                } else {
                    panic!("Database downgrade would be neccessary! Please confirm if you really want to do that.")
                }
            }
        }

        Ok(())
    }
}
