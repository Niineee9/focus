use chrono::{Datelike, Local, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// Bitmask de días: bit 0 = lunes … bit 6 = domingo.
pub const ALL_DAYS: u8 = 0b111_1111;
pub const WEEKDAYS: u8 = 0b001_1111;
pub const WEEKEND: u8 = 0b110_0000;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("base de datos: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("{0}")]
    Invalid(String),
}

impl Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub days_mask: u8,
    pub created_at: NaiveDate,
}

impl Category {
    pub fn is_scheduled(&self, date: NaiveDate) -> bool {
        is_scheduled(self.days_mask, date)
    }
}

pub fn is_scheduled(mask: u8, date: NaiveDate) -> bool {
    mask & (1 << date.weekday().num_days_from_monday()) != 0
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryInput {
    pub name: String,
    pub icon: String,
    pub color: String,
    pub days_mask: u8,
}

impl CategoryInput {
    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::Invalid("El nombre no puede estar vacío".into()));
        }
        if self.days_mask & ALL_DAYS == 0 {
            return Err(Error::Invalid("Elige al menos un día de la semana".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Completion {
    pub category_id: i64,
    pub date: NaiveDate,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        Self::init(Connection::open(path)?)
    }

    #[cfg(test)]
    pub fn in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS categories (
                 id         INTEGER PRIMARY KEY AUTOINCREMENT,
                 name       TEXT    NOT NULL,
                 icon       TEXT    NOT NULL DEFAULT '',
                 color      TEXT    NOT NULL,
                 days_mask  INTEGER NOT NULL,
                 created_at TEXT    NOT NULL,
                 position   INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE IF NOT EXISTS completions (
                 category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
                 date        TEXT    NOT NULL,
                 PRIMARY KEY (category_id, date)
             );",
        )?;
        let db = Self { conn };
        db.seed_defaults()?;
        Ok(db)
    }

    /// En el primer arranque crea las categorías de ejemplo.
    fn seed_defaults(&self) -> Result<()> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))?;
        if count == 0 {
            for (name, icon, color, mask) in [
                ("Ayuno", "🥗", "#1baf7a", ALL_DAYS),
                ("Ejercicio", "🏋️", "#2a78d6", WEEKDAYS),
                ("Inglés", "📚", "#eb6834", WEEKEND),
            ] {
                self.create_category(&CategoryInput {
                    name: name.into(),
                    icon: icon.into(),
                    color: color.into(),
                    days_mask: mask,
                })?;
            }
        }
        Ok(())
    }

    pub fn categories(&self) -> Result<Vec<Category>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, icon, color, days_mask, created_at
             FROM categories ORDER BY position, id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Category {
                id: r.get(0)?,
                name: r.get(1)?,
                icon: r.get(2)?,
                color: r.get(3)?,
                days_mask: r.get(4)?,
                created_at: r.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn create_category(&self, input: &CategoryInput) -> Result<Category> {
        input.validate()?;
        let today = Local::now().date_naive();
        self.conn.execute(
            "INSERT INTO categories (name, icon, color, days_mask, created_at, position)
             VALUES (?1, ?2, ?3, ?4, ?5, (SELECT COALESCE(MAX(position), 0) + 1 FROM categories))",
            params![input.name.trim(), input.icon, input.color, input.days_mask & ALL_DAYS, today],
        )?;
        Ok(Category {
            id: self.conn.last_insert_rowid(),
            name: input.name.trim().to_string(),
            icon: input.icon.clone(),
            color: input.color.clone(),
            days_mask: input.days_mask & ALL_DAYS,
            created_at: today,
        })
    }

    pub fn update_category(&self, id: i64, input: &CategoryInput) -> Result<()> {
        input.validate()?;
        let changed = self.conn.execute(
            "UPDATE categories SET name = ?1, icon = ?2, color = ?3, days_mask = ?4 WHERE id = ?5",
            params![input.name.trim(), input.icon, input.color, input.days_mask & ALL_DAYS, id],
        )?;
        if changed == 0 {
            return Err(Error::Invalid("La categoría no existe".into()));
        }
        Ok(())
    }

    pub fn delete_category(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM categories WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn completions_between(&self, from: NaiveDate, to: NaiveDate) -> Result<Vec<Completion>> {
        let mut stmt = self.conn.prepare(
            "SELECT category_id, date FROM completions WHERE date BETWEEN ?1 AND ?2",
        )?;
        let rows = stmt.query_map(params![from, to], |r| {
            Ok(Completion { category_id: r.get(0)?, date: r.get(1)? })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn completion_dates(&self, category_id: i64) -> Result<HashSet<NaiveDate>> {
        let mut stmt = self
            .conn
            .prepare("SELECT date FROM completions WHERE category_id = ?1")?;
        let rows = stmt.query_map([category_id], |r| r.get::<_, NaiveDate>(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Marca o desmarca un día. Devuelve `true` si quedó marcado.
    pub fn toggle_completion(&self, category_id: i64, date: NaiveDate) -> Result<bool> {
        if date > Local::now().date_naive() {
            return Err(Error::Invalid("No puedes marcar días futuros".into()));
        }
        let exists = self
            .conn
            .query_row(
                "SELECT 1 FROM completions WHERE category_id = ?1 AND date = ?2",
                params![category_id, date],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if exists {
            self.conn.execute(
                "DELETE FROM completions WHERE category_id = ?1 AND date = ?2",
                params![category_id, date],
            )?;
        } else {
            self.conn.execute(
                "INSERT INTO completions (category_id, date) VALUES (?1, ?2)",
                params![category_id, date],
            )?;
        }
        Ok(!exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_and_toggles() {
        let db = Db::in_memory().unwrap();
        let cats = db.categories().unwrap();
        assert_eq!(cats.len(), 3);
        let today = Local::now().date_naive();
        assert!(db.toggle_completion(cats[0].id, today).unwrap());
        assert_eq!(db.completion_dates(cats[0].id).unwrap().len(), 1);
        assert!(!db.toggle_completion(cats[0].id, today).unwrap());
        assert!(db.completion_dates(cats[0].id).unwrap().is_empty());
    }

    #[test]
    fn rejects_future_and_empty() {
        let db = Db::in_memory().unwrap();
        let id = db.categories().unwrap()[0].id;
        let tomorrow = Local::now().date_naive().succ_opt().unwrap();
        assert!(db.toggle_completion(id, tomorrow).is_err());
        let bad = CategoryInput { name: " ".into(), icon: "".into(), color: "#000".into(), days_mask: 1 };
        assert!(db.create_category(&bad).is_err());
    }
}
