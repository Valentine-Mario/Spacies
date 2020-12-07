use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::model::{Space, SpaceUser, User};
use crate::schema::events::dsl::space_id as event_space_id;
use crate::schema::events::dsl::*;
use crate::schema::spaces::dsl::*;
use crate::schema::spaces_users::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;
use actix::prelude::*;
use chrono::Local;
use cron::Schedule;
use std::{str::FromStr, time::Duration};

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

// Define actor
pub struct Scheduler;

// Provide Actor implementation for our actor
impl Actor for Scheduler {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("Actor is alive");

        ctx.run_later(duration_until_next(), move |this, ctx| {
            this.schedule_task(ctx)
        });
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        println!("Actor is stopped");
    }
}

impl Scheduler {
    fn schedule_task(&self, ctx: &mut Context<Self>) {
        println!("schedule_task event - {:?}", Local::now());
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        // create db connection pool
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool: Pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");

        run_job(pool);

        ctx.run_later(duration_until_next(), move |this, ctx| {
            this.schedule_task(ctx)
        });
    }
}

pub fn duration_until_next() -> Duration {
    let cron_expression = "0 0 0 * * * *"; //every 24 hrs
    let cron_schedule = Schedule::from_str(cron_expression).unwrap();
    let now = Local::now();
    let next = cron_schedule.upcoming(Local).next().unwrap();
    let duration_until = next.signed_duration_since(now);
    Duration::from_millis(duration_until.num_milliseconds() as u64)
}

fn run_job(db: Pool) {
    let conn = db.get().unwrap();
    let all_undone_events: Vec<Space> = spaces.load::<Space>(&conn).unwrap();
    println!("{:?}", all_undone_events);
}
