mod measure_units_mutation;
mod pipe_mutation;
mod pipe_stats_mutation;
mod production_plan_per_day_mutation;
mod sales_plan_per_day_mutation;
mod user_mutation;
mod production_info_mutation;

use async_graphql::Object;
use measure_units_mutation::MeasureUnitsMutation;
use pipe_mutation::PipeMutation;
use pipe_stats_mutation::PipeStatsMutation;
use production_plan_per_day_mutation::ProductionPlanPerDayMutation;
use sales_plan_per_day_mutation::SalesPlanPerDayMutation;
use user_mutation::UserMutation;
use production_info_mutation::ProductionInfoMutation;

pub struct MutationRoot;
#[Object]
impl MutationRoot {
    async fn users(&self) -> UserMutation {
        UserMutation
    }

    async fn pipe(&self) -> PipeMutation {
        PipeMutation
    }

    async fn pipe_stats(&self) -> PipeStatsMutation {
        PipeStatsMutation
    }

    async fn sales_plan_per_day(&self) -> SalesPlanPerDayMutation {
        SalesPlanPerDayMutation
    }

    async fn production_plan_per_day(&self) -> ProductionPlanPerDayMutation {
        ProductionPlanPerDayMutation
    }

    async fn measure_units(&self) -> MeasureUnitsMutation {
        MeasureUnitsMutation
    }

    async fn production_info(&self) -> ProductionInfoMutation {
        ProductionInfoMutation
    }
}
