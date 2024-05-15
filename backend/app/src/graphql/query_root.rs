mod measure_units_query;
mod pipe_query;
mod pipe_stats_query;
mod production_info_query;
mod production_plan_per_day_query;
mod sales_plan_per_day_query;
mod user_query;

use async_graphql::Object;
use production_info_query::ProductionInfoQuery;
use user_query::UserQuery;
use measure_units_query::MeasureUnitsQuery;
use pipe_query::PipeQuery;

use pipe_stats_query::PipeStatsQuery;
use production_plan_per_day_query::ProductionPlanPerDayQuery;
use sales_plan_per_day_query::SalesPlanPerDayQuery;

pub struct QueryRoot;
#[Object]
impl QueryRoot {
    /// API version - this is visible in the gql doc!
    async fn version(&self) -> &str {
        "1.0"
    }

    async fn users(&self) -> UserQuery {
        UserQuery
    }

    async fn production_info(&self) -> ProductionInfoQuery {
        ProductionInfoQuery
    }

    async fn measure_units(&self) -> MeasureUnitsQuery {
        MeasureUnitsQuery
    }

    async fn pipe(&self) -> PipeQuery {
        PipeQuery
    }

    async fn production_plan(&self) -> ProductionPlanPerDayQuery {
        ProductionPlanPerDayQuery
    }

    async fn sales_lan(&self) -> SalesPlanPerDayQuery {
        SalesPlanPerDayQuery
    }

    async fn pipe_stats(&self) -> PipeStatsQuery {
        PipeStatsQuery
    }
}
