use async_graphql::{Result, SimpleObject};
use common::{
    ctx::Ctx,
    error::{ApiError, ApiResult, Error},
};

use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use surrealdb::sql::{thing, Thing};

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, Eq, PartialEq)]
#[graphql(input_name = "Thing")]
pub struct ThingWrapper {
    #[graphql(skip)]
    thing_internal: Thing,
    thing: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, SimpleObject, Eq, PartialEq)]
#[graphql(input_name = "Things")]
pub struct Things {
    things: Vec<ThingWrapper>,
}

// use rand::Rng;
// impl Dummy<Faker> for ThingWrapper {
//     fn dummy_with_rng<R: Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
//         let id = Fake::fake_with_rng::<usize, _>(&(1000..2000), rng);
//     }
// }

pub trait ObjectWithThing: Send + Sync + std::fmt::Debug {
    fn thing(&self, ctx: &dyn Ctx) -> ApiResult<Thing>;

    // fn thing_or_variable<'a>(&'a self) -> ThingOrVariable<'a> {
    //     ThingOrVariable::Thing(self)
    // }
}

pub trait ObjectWithThings: Send + Sync {
    fn things(&self, ctx: &dyn Ctx) -> ApiResult<Vec<Thing>>;
}

impl ObjectWithThing for Thing {
    fn thing(&self, _ctx: &dyn Ctx) -> ApiResult<Thing> {
        Ok(self.clone())
    }
}

impl Deref for ThingWrapper {
    type Target = Thing;

    fn deref(&self) -> &Self::Target {
        &self.thing_internal
    }
}

impl DerefMut for ThingWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.thing_internal
    }
}

impl ObjectWithThing for String {
    fn thing(&self, ctx: &dyn Ctx) -> ApiResult<Thing> {
        thing(self).map_err(|_| ApiError {
            req_id: ctx.req_id(),
            error: Error::Generic {
                description: "Can't parse thing".to_string(),
            },
        })
    }
}

impl ObjectWithThings for Vec<String> {
    fn things(&self, ctx: &dyn Ctx) -> ApiResult<Vec<Thing>> {
        let mut result: Vec<Thing> = vec![];
        for i in self.iter() {
            let thing = i.thing(ctx)?;
            result.push(thing);
        }
        Ok(result)
    }
}

impl FromStr for ThingWrapper {
    type Err = &'static str;

    fn from_str(t: &str) -> Result<Self, Self::Err> {
        let thing = thing(t).map_err(|_| "error while parsing string")?;
        Ok(ThingWrapper {
            thing_internal: thing,
            thing: t.to_string(),
        })
    }
}

impl From<Thing> for ThingWrapper {
    fn from(t: Thing) -> Self {
        ThingWrapper {
            thing_internal: t.clone(),
            thing: t.to_string(),
        }
    }
}

impl From<ThingWrapper> for Thing {
    fn from(t: ThingWrapper) -> Thing {
        t.thing_internal
    }
}

// impl From<ThingWrapper> for Option<Thing> {
//     fn from(t: ThingWrapper) -> Option<Thing> {
//         t.thing_internal
//     }
// }

impl ObjectWithThing for ThingWrapper {
    fn thing(&self, ctx: &dyn Ctx) -> ApiResult<Thing> {
        //self.id_thing(ctx)
        self.thing.thing(ctx)
    }
}

impl ThingWrapper {
    //pub fn id_thing(&self, ctx: &dyn Ctx) -> ApiResult<Thing> {
    //    self.thing_internal.clone().ok_or(ApiError {
    //        req_id: ctx.req_id(),
    //        error: Error::Generic {
    //            description: format!("Can't get id for Thing from {:?}", self.thing_internal)
    //                .to_string(),
    //        },
    //    })
    //}

    pub fn from_option_thing(thing: Option<Thing>, ctx: &dyn Ctx) -> ApiResult<ThingWrapper> {
        Ok(Self::from(thing.clone().ok_or(ApiError {
            req_id: ctx.req_id(),
            error: Error::Generic {
                description: format!("Can't get id for Thing from {:?}", thing.clone()).to_string(),
            },
        })?))
    }

    // pub fn option_id_thing(thing: Option<Self>, ctx: &dyn Ctx) -> ApiResult<Thing> {
    //     thing
    //         .clone()
    //         .ok_or(ApiError {
    //             req_id: ctx.req_id(),
    //             error: Error::Generic {
    //                 description: format!("Can't get id for Thing from {:?}", thing.clone())
    //                     .to_string(),
    //             },
    //         })?
    //         .thing(&ctx)
    // }
}
