use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::helpers::{email, email_template};
use crate::model::{Event, Space, SpaceUser, User};
use crate::schema::events::dsl::*;
use crate::schema::spaces::dsl::*;
use crate::schema::users::dsl::*;
use crate::Pool;
use actix::prelude::*;
use chrono::prelude::*;
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

//cron job to remin space members of any set event
fn run_job(db: Pool) {
    let conn = db.get().unwrap();
    println!("Running daily job...");
    //get all pending events
    let items: Vec<Event> = events
        .filter(reminded.eq(false))
        .load::<Event>(&conn)
        .unwrap();

    //get today's date
    let today: DateTime<Local> = Local::now();
    let other_email_address = std::env::var("EMAIL_ADDRESS").expect("EMAIL ADDRESS not set");
    let other_email_password = std::env::var("EMAIL_PASSWORD").expect("EMAIL PASSWORD not set");
    let other_email_provider=std::env::var("EMAIL_PROVIDER").expect("EMAIL PROVIDER not set");

    for val in items.iter() {
        //get date of the current event
        let set_event_date: NaiveDateTime = val.event_date;
        if (today.year(), today.month(), today.day())
            == (
                set_event_date.year(),
                set_event_date.month(),
                set_event_date.day(),
            )
        {
            let _update_event = diesel::update(events.find(val.id))
                .set(reminded.eq(&true))
                .execute(&conn);

            //get all users in event channel
            let space = spaces.find(val.space_id).first::<Space>(&conn).unwrap();
            let user_spaces: Vec<_> = SpaceUser::belonging_to(&space)
                .inner_join(users)
                .load::<(SpaceUser, User)>(&conn)
                .unwrap();

            //send email to all membrs
            let template = email_template::send_reminder(
                &val.event_name,
                &space.spaces_name,
                &val.event_description,
            );

            for a in user_spaces.iter() {
                email::send_email(
                    &a.1.email,
                    &a.1.username,
                    &"Event reminder".to_string(),
                    &template,
                    &other_email_address,
                    &other_email_password,
                    &other_email_provider
                );
            }
        }
    }
}
