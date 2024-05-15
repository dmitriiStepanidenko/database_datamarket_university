use async_graphql::{Context, Object, Result};
use common::ctx::CtxStruct;
use db::Db;

use service::{
    measure_units::{
        CreateMeasureUnitsTypeInput, MeasureUnits, MeasureUnitsUseCases,
    },
    thing_derived::ThingDerived,
};

pub struct MeasureUnitsMutation;
#[Object]
impl MeasureUnitsMutation {
    async fn create(
        &self,
        ctx: &Context<'_>,
        ct_input: CreateMeasureUnitsTypeInput,
    ) -> Result<MeasureUnits> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(MeasureUnitsUseCases::create(ct_input, db, ctx).await?)
    }

    async fn update(
        &self,
        ctx: &Context<'_>,
        ct_input: CreateMeasureUnitsTypeInput,
        id: ThingDerived,
    ) -> Result<MeasureUnits> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(MeasureUnitsUseCases::update(ct_input, &id, db, ctx).await?)
    }
}
