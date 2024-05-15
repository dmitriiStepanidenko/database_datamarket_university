use crate::{
    datetime::DateTimeDerived,
    machinery_type::{CreateMachineryTypeInput, MachineryType, MachineryTypeUseCases},
    measure_units::{CreateMeasureUnitsTypeInput, MeasureUnits, MeasureUnitsUseCases},
    pipe::{CreatePipeInput, Pipe, PipeUseCases},
    pipe_stats::{CreatePipeStatsInput, PipeStats, PipeStatsUseCases},
    pipe_type::{CreatePipeTypeInput, PipeType, PipeTypeUseCases},
    production_info::ProductionInfoUseCases,
    production_per_day::{CreateProductionPlanPerDayTypeInput, ProductionPlanPerDayUseCases},
    sales_per_day::{CreateSalesPlanPerDayTypeInput, SalesPlanPerDayUnitsUseCases},
    thing_derived::ThingDerived,
    thing_wrapper::ObjectWithThing,
};

#[cfg(not(debug_assertions))]
lazy_static::lazy_static! {
    static ref ADMIN_USER_EMAIL: String =
        std::env::var("ADMIN_USER_EMAIL").expect("SHOULD PROVIDE ADMIN_USER_EMAIL");
    static ref ADMIN_PASSWORD: Option<String> =
        std::env::var("ADMIN_PASSWORD").ok();
}

#[cfg(debug_assertions)]
lazy_static::lazy_static! {
    static ref ADMIN_USER_EMAIL: String = "test@test.com".to_string();
    static ref ADMIN_PASSWORD: Option<String> = Some("pass".to_string());
}

pub async fn check_and_recreate_admin_user() {
    use crate::service::user::{CreateUserInput, User, UserUseCases};
    use common::{ctx::MockCtx, role::Role};
    use db::DB;
    let mut ctx = MockCtx::new();
    ctx.expect_req_id().return_const(uuid::Uuid::new_v4());

    let user: Vec<User> = DB
        .query("SELECT * FROM User WHERE email = $email;")
        .bind(("email", ADMIN_USER_EMAIL.clone().as_str()))
        .await
        .expect("Failed to connect to db while checking admin user")
        .take::<Vec<User>>(0)
        .expect("Failed while checking existence of admin user");
    println!("User: {:?}", user);
    if user.is_empty() {
        let mut user_adm = UserUseCases::create_user(
            CreateUserInput {
                email: ADMIN_USER_EMAIL.clone(),
                password: "pass".to_string(),
            },
            &DB,
            &ctx,
        )
        .await
        .expect("Problem with creating user");
        user_adm.roles.push(Role::Admin);
        let _user_adm =
            UserUseCases::update_user(user_adm.clone(), user_adm.clone().id.unwrap(), &DB, &ctx)
                .await
                .expect("Problem with givin to user adm priviledges");
    }
}

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use common::{ctx::Ctx, ApiResult};
use db::Db;
use rust_decimal::Decimal;
use surrealdb::sql::Datetime;
pub async fn machinery_type_shortcut(
    name: &str,
    units: &dyn ObjectWithThing,
    db: &Db,
    ctx: &dyn Ctx,
) -> ApiResult<MachineryType> {
    MachineryTypeUseCases::create(
        CreateMachineryTypeInput {
            name: name.to_string(),
            max_flow: Decimal::new(1000000, 0),
            wearout_max: Decimal::new(1000000, 0),
            units: ThingDerived::from(units.thing(ctx)?),
        },
        db,
        ctx,
    )
    .await
}

pub async fn pipe_type_shortcut(
    name: &str,
    units: &dyn ObjectWithThing,
    db: &Db,
    ctx: &dyn Ctx,
) -> ApiResult<PipeType> {
    PipeTypeUseCases::create(
        CreatePipeTypeInput {
            name: name.to_string(),
            max_flow: Decimal::new(1000000, 0),
            wearout_max: Decimal::new(1000000, 0),
            units: ThingDerived::from(units.thing(ctx)?),
        },
        db,
        ctx,
    )
    .await
}

pub async fn measure_units_shortcut(name: &str, db: &Db, ctx: &dyn Ctx) -> ApiResult<MeasureUnits> {
    MeasureUnitsUseCases::create(
        CreateMeasureUnitsTypeInput {
            name: name.to_string(),
        },
        db,
        ctx,
    )
    .await
}

pub async fn pipe_stats_shortcut(
    date: DateTimeDerived,
    flow: Decimal,
    wearout: Decimal,
    units: &MeasureUnits,
    pipe: &Pipe,
    db: &Db,
    ctx: &dyn Ctx,
) -> ApiResult<PipeStats> {
    PipeStatsUseCases::create(
        CreatePipeStatsInput {
            pipe: pipe.id.clone().unwrap(),
            units: units.id.clone().unwrap(),
            wearout,
            date,
            flow,
        },
        db,
        ctx,
    )
    .await
}

pub async fn seed_data() {
    use crate::service::{
        measure_units,
        raw_material::{CreateRawMaterialInput, RawMaterialUseCases},
    };
    use common::{ctx::MockCtx, role::Role};
    use db::DB;
    let mut ctx = MockCtx::new();
    ctx.expect_req_id().return_const(uuid::Uuid::new_v4());
    // Вход:
    // - Просеенный сахар
    // - Очищенное масло
    // - Фильтрованное цельное молоко
    // - Фильтрованные сливки
    //
    // 1. Составление смеси - Изначальная смесь
    // 2. Фильтрация смеси - Фильтрованная смесь
    // 3. Пастеризация смеси - Пастеризованная смесь
    // 4. Гомогенизация смеси - Гомогенизированная смесь
    // 5. Охлаждение смеси - Охлажденная смесь
    // 6. Созревание смеси - Созревшая смесь
    // 7. Фризерование - Мороженое
    // 8. Фасование - Брикет мороженого
    // 9. Упаковывание - Палет мороженого
    //
    // Выход: палеты с мороженым

    let sugar = RawMaterialUseCases::create(
        CreateRawMaterialInput {
            name: "Просеннай сахар".to_string(),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let butter = RawMaterialUseCases::create(
        CreateRawMaterialInput {
            name: "Очищенное масло".to_string(),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let milk = RawMaterialUseCases::create(
        CreateRawMaterialInput {
            name: "Фильтрованное цельное молоко".to_string(),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let cream = RawMaterialUseCases::create(
        CreateRawMaterialInput {
            name: "Фильтрованные сливки".to_string(),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let initial_mix = RawMaterialUseCases::create(
        CreateRawMaterialInput {
            name: "Изначальная смесь".to_string(),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let filtered_mix = RawMaterialUseCases::create(
        CreateRawMaterialInput {
            name: "Фильтрованная смесь".to_string(),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let pasteurized_mix = RawMaterialUseCases::create(
        CreateRawMaterialInput {
            name: "Пастеризованная смесь".to_string(),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let homogenized_mix = RawMaterialUseCases::create(
        CreateRawMaterialInput {
            name: "Гомогенизованная смесь".to_string(),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let cooled_mix = RawMaterialUseCases::create(
        CreateRawMaterialInput {
            name: "Охлажденная смесь".to_string(),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let matured_mix = RawMaterialUseCases::create(
        CreateRawMaterialInput {
            name: "Созревшая смесь".to_string(),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let ice_cream = RawMaterialUseCases::create(
        CreateRawMaterialInput {
            name: "Мороженое".to_string(),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let ice_cream_briquette = RawMaterialUseCases::create(
        CreateRawMaterialInput {
            name: "Брикет мороженого".to_string(),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let ice_cream_pallet = RawMaterialUseCases::create(
        CreateRawMaterialInput {
            name: "Палет мороженого".to_string(),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();

    let m_3 = measure_units_shortcut("м^3", &DB, &ctx).await.unwrap();
    let briquette = measure_units_shortcut("брикет", &DB, &ctx).await.unwrap();
    let palette = measure_units_shortcut("палет", &DB, &ctx).await.unwrap();

    let production_plan_per_day0 = ProductionPlanPerDayUseCases::create(
        CreateProductionPlanPerDayTypeInput {
            amount: Decimal::new(100, 0),
            units: palette.id.clone().unwrap(),
            date: DateTimeDerived(
                DateTime::from_utc(
                    NaiveDate::from_ymd_opt(2024, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap(),
                    Utc,
                )
                .into(),
            ),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();

    let production_plan_per_day1 = ProductionPlanPerDayUseCases::create(
        CreateProductionPlanPerDayTypeInput {
            amount: Decimal::new(105, 0),
            units: palette.id.clone().unwrap(),
            date: DateTimeDerived(
                DateTime::from_utc(
                    NaiveDate::from_ymd_opt(2024, 1, 2)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap(),
                    Utc,
                )
                .into(),
            ),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();

    let sales_plan_per_day0 = SalesPlanPerDayUnitsUseCases::create(
        CreateSalesPlanPerDayTypeInput {
            amount: Decimal::new(105, 0),
            units: palette.id.clone().unwrap(),
            date: DateTimeDerived(
                DateTime::from_utc(
                    NaiveDate::from_ymd_opt(2024, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap(),
                    Utc,
                )
                .into(),
            ),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();

    let sales_plan_per_day1 = SalesPlanPerDayUnitsUseCases::create(
        CreateSalesPlanPerDayTypeInput {
            amount: Decimal::new(105, 0),
            units: palette.id.clone().unwrap(),
            date: DateTimeDerived(
                DateTime::from_utc(
                    NaiveDate::from_ymd_opt(2024, 1, 2)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap(),
                    Utc,
                )
                .into(),
            ),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();

    let pipe_type = pipe_type_shortcut("Конвейер с палетами мороженого", &palette, &DB, &ctx)
        .await
        .unwrap();

    let final_pipe = PipeUseCases::create(
        CreatePipeInput {
            name: "Конвейер с палетами мороженого № 1".to_string(),
            material: ice_cream_pallet.id.clone().unwrap(),
            pipe_type: pipe_type.id.clone().unwrap(),
        },
        &DB,
        &ctx,
    )
    .await
    .unwrap();

    // Теперь необходимо добавить pipe_stats для данной сущности
    // pipe_stats считается по изменению

    let wearout = Decimal::new(1002, 1);

    let date_0 = Utc.with_ymd_and_hms(2024, 1, 1, 1, 0, 0).unwrap();
    let flow_0 = Decimal::new(5, 1); // = 0.5

    let date_1 = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
    let flow_1 = Decimal::new(6, 0);

    let date_2 = Utc.with_ymd_and_hms(2024, 1, 1, 20, 0, 0).unwrap();
    let flow_2 = Decimal::new(8, 0);

    // Итого за первый день: 0.5*9+6*10+4*8 = 96.5

    let date_3 = Utc.with_ymd_and_hms(2024, 1, 2, 10, 0, 0).unwrap();
    let flow_3 = Decimal::new(1, 0);

    let date_4 = Utc.with_ymd_and_hms(2024, 1, 2, 22, 0, 0).unwrap();
    let flow_4 = Decimal::new(20, 0);

    // Итого за второй день: 8*10 + 1*12 + 20*2 = 132

    let _ = pipe_stats_shortcut(
        date_0.into(),
        flow_0,
        wearout,
        &palette,
        &final_pipe,
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let _ = pipe_stats_shortcut(
        date_1.into(),
        flow_1,
        wearout,
        &palette,
        &final_pipe,
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let _ = pipe_stats_shortcut(
        date_2.into(),
        flow_2,
        wearout,
        &palette,
        &final_pipe,
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let _ = pipe_stats_shortcut(
        date_3.into(),
        flow_3,
        wearout,
        &palette,
        &final_pipe,
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let _ = pipe_stats_shortcut(
        date_4.into(),
        flow_4,
        wearout,
        &palette,
        &final_pipe,
        &DB,
        &ctx,
    )
    .await
    .unwrap();

    let production_info_0 = ProductionInfoUseCases::select_create(
        date_1.into(),
        Some(final_pipe.thing(&ctx).unwrap().into()),
        &DB,
        &ctx,
    )
    .await
    .unwrap();
    let production_info_1 = ProductionInfoUseCases::select_create(
        date_3.into(),
        Some(final_pipe.thing(&ctx).unwrap().into()),
        &DB,
        &ctx,
    )
    .await
    .unwrap();
}
