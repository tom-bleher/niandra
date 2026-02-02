//! Query filter utilities for building parameterized SQL queries.

use duckdb::ToSql;

/// Date range filter for queries.
#[derive(Debug, Clone, Default)]
pub struct DateFilter<'a> {
    pub start: Option<&'a str>,
    pub end: Option<&'a str>,
}

impl<'a> DateFilter<'a> {
    /// Create a new date filter.
    pub fn new(start: Option<&'a str>, end: Option<&'a str>) -> Self {
        Self { start, end }
    }

    /// Append date filter clauses to a query string.
    /// DuckDB handles timezone conversion automatically when comparing timestamps.
    pub fn apply(&self, query: &mut String, params: &mut Vec<String>) {
        if let Some(start) = self.start {
            query.push_str(" AND timestamp >= ?");
            params.push(start.to_string());
        }
        if let Some(end) = self.end {
            query.push_str(" AND timestamp <= ?");
            params.push(end.to_string());
        }
    }

    /// Convert string params to a vector of references suitable for DuckDB.
    pub fn params_as_refs(params: &[String]) -> Vec<&dyn ToSql> {
        params.iter().map(|s| s as &dyn ToSql).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_filter_none() {
        let filter = DateFilter::new(None, None);
        let mut query = "SELECT * FROM plays WHERE 1=1".to_string();
        let mut params = Vec::new();
        filter.apply(&mut query, &mut params);

        assert_eq!(query, "SELECT * FROM plays WHERE 1=1");
        assert!(params.is_empty());
    }

    #[test]
    fn test_date_filter_start_only() {
        let filter = DateFilter::new(Some("2024-01-01"), None);
        let mut query = "SELECT * FROM plays WHERE 1=1".to_string();
        let mut params = Vec::new();
        filter.apply(&mut query, &mut params);

        assert_eq!(
            query,
            "SELECT * FROM plays WHERE 1=1 AND timestamp >= ?"
        );
        assert_eq!(params, vec!["2024-01-01"]);
    }

    #[test]
    fn test_date_filter_both() {
        let filter = DateFilter::new(Some("2024-01-01"), Some("2024-12-31"));
        let mut query = "SELECT * FROM plays WHERE 1=1".to_string();
        let mut params = Vec::new();
        filter.apply(&mut query, &mut params);

        assert_eq!(
            query,
            "SELECT * FROM plays WHERE 1=1 AND timestamp >= ? AND timestamp <= ?"
        );
        assert_eq!(params, vec!["2024-01-01", "2024-12-31"]);
    }
}
