use crate::entities;
use anyhow::Context;

use super::Repository;

impl Repository {
    pub async fn find_tables(&self) -> anyhow::Result<Vec<entities::Table>> {
        const QUERY: &str = "select
                               table_name as name,
                               (xpath('/row/cnt/text()', xml_count))[1]::text::int as row_count
                             from (
                               select table_name, table_schema,
                                 query_to_xml(format('select count(*) as cnt from %I.%I', table_schema, table_name), false, true, '') as xml_count
                               from information_schema.tables
                               where table_schema = 'public'
                             ) t";

        sqlx::query_as::<_, entities::Table>(QUERY)
            .fetch_all(&self.pool)
            .await
            .context("[sql] get_tables()")
    }
}
